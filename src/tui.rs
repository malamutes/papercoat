use crate::config::Config;
use crate::extractor::extract_text;
use crate::renderer;
use crate::templates;
use crate::transformer::{self, PaperData};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use std::path::{Path, PathBuf};

const PURPLE: Color = Color::Indexed(99);
const INDIGO: Color = Color::Indexed(69);

pub struct App {
    // Navigation
    cwd: PathBuf,
    entries: Vec<PathBuf>,
    selected_index: usize,
    selected_file: Option<PathBuf>,

    // Paper data
    paper: Option<PaperData>,
    raw_text: Option<String>,

    // Options
    template_index: usize,
    author_override: String,
    output_format: usize, // 0 = tex, 1 = pdf

    // Status
    status: String,
    generating: bool,
    done: bool,
    mode: AppMode,
}

#[derive(PartialEq)]
enum AppMode {
    Browse,
    Preview,
}

static FORMATS: &[&str] = &["LaTeX (.tex)", "PDF (via LaTeX)"];

impl App {
    pub fn new() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let mut app = Self {
            cwd: cwd.clone(),
            entries: Vec::new(),
            selected_index: 0,
            selected_file: None,
            paper: None,
            raw_text: None,
            template_index: 0,
            author_override: String::new(),
            output_format: 0,
            status: String::new(),
            generating: false,
            done: false,
            mode: AppMode::Browse,
        };
        app.refresh_dir();
        Ok(app)
    }

    fn refresh_dir(&mut self) {
        let mut entries = Vec::new();
        if let Ok(read_dir) = std::fs::read_dir(&self.cwd) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let is_pdf = path.extension().map(|e| e == "pdf").unwrap_or(false);
                if path.is_dir() || is_pdf {
                    entries.push(path);
                }
            }
        }
        entries.sort_by(|a, b| {
            let a_dir = a.is_dir();
            let b_dir = b.is_dir();
            if a_dir != b_dir {
                b_dir.cmp(&a_dir) // dirs first
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });
        self.entries = entries;
        self.selected_index = 0;
    }

    fn enter_dir(&mut self, dir: &Path) {
        self.cwd = dir.to_path_buf();
        self.refresh_dir();
        self.mode = AppMode::Browse;
        self.status = format!("📂 {}", dir.display());
    }

    fn go_up(&mut self) {
        let parent = self.cwd.parent().map(|p| p.to_path_buf());
        if let Some(dir) = parent {
            self.enter_dir(&dir);
        }
    }

    fn select_current(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let entry = self.entries[self.selected_index].clone();
        if entry.is_dir() {
            self.enter_dir(&entry);
        } else {
            self.selected_file = Some(entry.clone());
            self.status = format!(
                "📄 Selected: {}",
                entry.file_name().unwrap().to_string_lossy()
            );
            self.mode = AppMode::Preview;
            self.generate_paper(Some(&entry));
        }
    }

    fn generate_paper(&mut self, path_opt: Option<&PathBuf>) {
        let path = match path_opt.or(self.selected_file.as_ref()) {
            Some(p) => p.clone(),
            None => return,
        };

        self.generating = true;
        self.status = "⏳ Extracting text...".to_string();

        match extract_text(&path, false) {
            Ok(extracted) => {
                self.status = "⏳ Transforming into academic paper...".to_string();
                let cfg = Config::load();
                let author = if self.author_override.is_empty() {
                    cfg.author.clone()
                } else {
                    Some(self.author_override.clone())
                };

                let page_count = crate::extractor::estimate_page_count(&path);
                let paper = transformer::transform_text(&extracted.text, None, author, page_count);
                self.paper = Some(paper);
                self.raw_text = Some(extracted.text);
                self.status = format!(
                    "✅ Generated: {} words, {} sections",
                    extracted.word_count,
                    self.paper.as_ref().map(|p| p.sections.len()).unwrap_or(0),
                );
            }
            Err(e) => {
                self.status = format!("❌ Error: {}", e);
            }
        }
        self.generating = false;
    }

    fn generate_output(&mut self) -> Result<()> {
        let paper = match &self.paper {
            Some(p) => p,
            None => {
                self.status = "⚠️ No paper data. Select a PDF first.".to_string();
                return Ok(());
            }
        };

        let path = match &self.selected_file {
            Some(p) => p.clone(),
            None => {
                self.status = "⚠️ No file selected.".to_string();
                return Ok(());
            }
        };

        let template = templates::list()[self.template_index].name;
        let fmt = if self.output_format == 0 {
            "tex"
        } else {
            "pdf"
        };
        let output_tex = path.with_file_name(format!(
            "{}_academic.tex",
            path.file_stem().unwrap().to_string_lossy(),
        ));
        let output_path = if fmt == "pdf" {
            path.with_file_name(format!(
                "{}_academic.pdf",
                path.file_stem().unwrap().to_string_lossy(),
            ))
        } else {
            output_tex.clone()
        };

        self.status = "⏳ Generating LaTeX...".to_string();
        renderer::render_tex(paper, &output_tex, template)?;

        if fmt == "pdf" {
            self.status = "⏳ Compiling with pdflatex...".to_string();
            match renderer::compile_latex(&output_tex)? {
                Some(_) => {
                    self.status = format!(
                        "✅ Saved: {}.pdf",
                        output_path.file_stem().unwrap().to_string_lossy()
                    );
                }
                None => {
                    self.status = format!(
                        "✅ Saved: {} (no pdflatex)",
                        output_tex.file_name().unwrap().to_string_lossy()
                    );
                }
            }
        } else {
            self.status = format!(
                "✅ Saved: {}",
                output_tex.file_name().unwrap().to_string_lossy()
            );
        }

        self.done = true;
        Ok(())
    }
}

pub fn run_tui() -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut app = App::new()?;
    let tick_rate = std::time::Duration::from_millis(100);

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => {
                            if app.mode == AppMode::Preview {
                                app.mode = AppMode::Browse;
                                app.status.clear();
                            }
                        }
                        _ => handle_key(key.code, &mut app)?,
                    }
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn handle_key(code: KeyCode, app: &mut App) -> Result<()> {
    match app.mode {
        AppMode::Browse => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if app.selected_index > 0 {
                    app.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.selected_index + 1 < app.entries.len() {
                    app.selected_index += 1;
                }
            }
            KeyCode::Enter => app.select_current(),
            KeyCode::Backspace => app.go_up(),
            KeyCode::Char('g') => {
                app.selected_index = 0;
            }
            KeyCode::Char('G') if !app.entries.is_empty() => {
                app.selected_index = app.entries.len() - 1;
            }
            _ => {}
        },
        AppMode::Preview => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if app.template_index > 0 {
                    app.template_index -= 1;
                    if app.selected_file.is_some() {
                        app.generate_paper(None);
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = templates::list().len().saturating_sub(1);
                if app.template_index < max {
                    app.template_index += 1;
                    if app.selected_file.is_some() {
                        app.generate_paper(None);
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if app.output_format > 0 {
                    app.output_format -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if app.output_format + 1 < FORMATS.len() {
                    app.output_format += 1;
                }
            }
            KeyCode::Enter => {
                app.generate_output()?;
            }
            KeyCode::Char('r') => {
                if app.selected_file.is_some() {
                    app.generate_paper(None);
                }
            }
            KeyCode::Char('a') => {
                // TODO: open text input for author override
            }
            _ => {}
        },
    }
    Ok(())
}

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title bar
            Constraint::Min(1),    // main content
            Constraint::Length(4), // options bar
            Constraint::Length(1), // status bar
        ])
        .split(area);

    draw_title(frame, main_layout[0]);
    match app.mode {
        AppMode::Browse => draw_browser(frame, main_layout[1], app),
        AppMode::Preview => draw_preview(frame, main_layout[1], app),
    }
    draw_options(frame, main_layout[2], app);
    draw_status(frame, main_layout[3], app);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "  PaperCoat ",
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled("✦  ", Style::new().fg(PURPLE)),
        Span::styled(
            "Turn PDFs into academic papers",
            Style::new().fg(Color::Gray),
        ),
        Span::styled("  [", Style::new().fg(Color::DarkGray)),
        Span::styled("q", Style::new().fg(Color::Green)),
        Span::styled("uit]", Style::new().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::new().fg(PURPLE)),
    );
    frame.render_widget(title, area);
}

fn draw_browser(frame: &mut Frame, area: Rect, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let current_path = app.cwd.to_string_lossy().to_string();

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let name = entry.file_name().unwrap().to_string_lossy();
            let prefix = if entry.is_dir() { "📁 " } else { "📄 " };
            let style = if i == app.selected_index {
                Style::new()
                    .bg(INDIGO)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir() {
                Style::new().fg(PURPLE)
            } else {
                Style::new().fg(Color::White)
            };
            ListItem::new(format!("{}{}", prefix, name)).style(style)
        })
        .collect();

    let files_list = List::new(items)
        .block(
            Block::default()
                .title(format!(" 📂 {} ", &current_path))
                .borders(Borders::ALL)
                .border_style(Style::new().fg(PURPLE))
                .title_style(Style::new().fg(Color::Cyan)),
        )
        .highlight_style(Style::new().bg(INDIGO).fg(Color::White));

    frame.render_widget(files_list, layout[0]);

    // Help panel
    let help_lines = vec![
        Line::from(vec![
            Span::styled("↑/↓ or j/k  ", Style::new().fg(Color::Green)),
            Span::raw("Navigate"),
        ]),
        Line::from(vec![
            Span::styled("Enter       ", Style::new().fg(Color::Green)),
            Span::raw("Open dir / Select PDF"),
        ]),
        Line::from(vec![
            Span::styled("Backspace   ", Style::new().fg(Color::Green)),
            Span::raw("Go up"),
        ]),
        Line::from(vec![
            Span::styled("g/G         ", Style::new().fg(Color::Green)),
            Span::raw("Top/Bottom"),
        ]),
        Line::from(vec![
            Span::styled("q           ", Style::new().fg(Color::Green)),
            Span::raw("Quit"),
        ]),
    ];
    let help = Paragraph::new(Text::from(help_lines)).block(
        Block::default()
            .title(" ⌨️  Controls ")
            .borders(Borders::ALL)
            .border_style(Style::new().fg(Color::DarkGray)),
    );
    frame.render_widget(help, layout[1]);
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Left: Paper metadata
    let mut preview_lines = Vec::new();

    if let Some(paper) = &app.paper {
        preview_lines.push(Line::from(vec![
            Span::styled(
                "Title: ",
                Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
            ),
            Span::raw(&paper.title),
        ]));
        preview_lines.push(Line::from(vec![
            Span::styled(
                "Authors: ",
                Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
            ),
            Span::raw(&paper.author_line),
        ]));
        preview_lines.push(Line::from(vec![
            Span::styled(
                "Words: ",
                Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", paper.word_count)),
        ]));
        preview_lines.push(Line::from(vec![
            Span::styled(
                "Refs: ",
                Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", paper.references.len())),
        ]));
        preview_lines.push(Line::from(""));
        preview_lines.push(Line::from(vec![Span::styled(
            "Abstract",
            Style::new().fg(INDIGO).add_modifier(Modifier::BOLD),
        )]));
        preview_lines.push(Line::from(vec![
            Span::styled("  Background: ", Style::new().fg(Color::DarkGray)),
            Span::raw(truncate(&paper.abstract_sections.background, 120)),
        ]));
        preview_lines.push(Line::from(vec![
            Span::styled("  Methods: ", Style::new().fg(Color::DarkGray)),
            Span::raw(truncate(&paper.abstract_sections.methods, 120)),
        ]));
        preview_lines.push(Line::from(vec![
            Span::styled("  Results: ", Style::new().fg(Color::DarkGray)),
            Span::raw(truncate(&paper.abstract_sections.results, 120)),
        ]));
        preview_lines.push(Line::from(""));
        preview_lines.push(Line::from(vec![Span::styled(
            "Sections",
            Style::new().fg(INDIGO).add_modifier(Modifier::BOLD),
        )]));
        for section in &paper.sections {
            preview_lines.push(Line::from(vec![Span::styled(
                format!("  📄 {}", section.heading),
                Style::new().fg(Color::White),
            )]));
        }
    } else if app.generating {
        preview_lines.push(Line::from(vec![Span::styled(
            "  Generating...",
            Style::new().fg(Color::Yellow),
        )]));
    } else {
        preview_lines.push(Line::from(vec![Span::styled(
            "  Select a PDF to preview",
            Style::new().fg(Color::DarkGray),
        )]));
    }

    let preview = Paragraph::new(Text::from(preview_lines))
        .block(
            Block::default()
                .title(" 📋  Paper Preview ")
                .borders(Borders::ALL)
                .border_style(Style::new().fg(INDIGO)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, layout[0]);

    // Right: Controls
    let templates_list = templates::list();
    let mut control_lines = vec![Line::from(vec![Span::styled(
        "Template",
        Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
    )])];
    for (i, t) in templates_list.iter().enumerate() {
        let marker = if i == app.template_index {
            "▸ "
        } else {
            "  "
        };
        let style = if i == app.template_index {
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::DarkGray)
        };
        control_lines.push(Line::from(vec![
            Span::styled(marker, Style::new().fg(INDIGO)),
            Span::styled(t.name, style),
        ]));
    }
    control_lines.push(Line::from(""));
    control_lines.push(Line::from(vec![Span::styled(
        "Format",
        Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
    )]));
    for (i, f) in FORMATS.iter().enumerate() {
        let marker = if i == app.output_format { "▸ " } else { "  " };
        let style = if i == app.output_format {
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::DarkGray)
        };
        control_lines.push(Line::from(vec![
            Span::styled(marker, Style::new().fg(INDIGO)),
            Span::styled(*f, style),
        ]));
    }

    let controls = Paragraph::new(Text::from(control_lines)).block(
        Block::default()
            .title(" ⚙️  Options ")
            .borders(Borders::ALL)
            .border_style(Style::new().fg(PURPLE)),
    );
    frame.render_widget(controls, layout[1]);
}

fn draw_options(frame: &mut Frame, area: Rect, app: &App) {
    let help = match app.mode {
        AppMode::Browse => "↑↓ Navigate  |  Enter Select  |  Backspace Up  |  q Quit".to_string(),
        AppMode::Preview => {
            "↑↓ Template  |  ←→ Format  |  Enter Generate  |  r Regenerate  |  Esc Back  |  q Quit"
                .to_string()
        }
    };

    let file_name = match &app.selected_file {
        Some(p) => p.file_name().unwrap().to_string_lossy().to_string(),
        None => "No file selected".to_string(),
    };

    let options_text = Paragraph::new(Line::from(vec![
        Span::styled("  File: ", Style::new().fg(Color::DarkGray)),
        Span::styled(file_name, Style::new().fg(Color::Cyan)),
        Span::styled("  │  ", Style::new().fg(Color::DarkGray)),
        Span::styled(&help, Style::new().fg(Color::Gray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(PURPLE)),
    );
    frame.render_widget(options_text, area);
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let status = if app.status.is_empty() {
        "Ready".to_string()
    } else {
        app.status.clone()
    };
    let status_style = if app.status.starts_with('✅') {
        Style::new().fg(Color::Green)
    } else if app.status.starts_with('❌') || app.status.starts_with("⚠️") {
        Style::new().fg(Color::Red)
    } else if app.status.starts_with("⏳") {
        Style::new().fg(Color::Yellow)
    } else {
        Style::new().fg(Color::DarkGray)
    };

    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::new().fg(Color::DarkGray)),
        Span::styled(status, status_style),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(PURPLE)),
    );
    frame.render_widget(status_bar, area);
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s
    }
}
