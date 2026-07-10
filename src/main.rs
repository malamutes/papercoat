mod cli;
mod config;
mod extractor;
mod renderer;
mod templates;
mod transformer;
mod tui;

use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::path::{Path, PathBuf};

const PURPLE: u8 = 99;
const INDIGO: u8 = 69;

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    // Launch TUI if no input and no action flags
    if args.input.is_none() && !args.list_templates && !args.show_config
        && args.config_set.is_none() && !args.config_reset
    {
        return tui::run_tui();
    }

    if args.list_templates {
        print_templates();
        return Ok(());
    }

    if args.show_config {
        print_config();
        return Ok(());
    }

    if let Some(kv) = &args.config_set {
        if kv.len() == 2 {
            let mut cfg = config::Config::load();
            match kv[0].as_str() {
                "template" => cfg.template = kv[1].clone(),
                "format" => cfg.format = kv[1].clone(),
                "title" => cfg.title = Some(kv[1].clone()),
                "author" => cfg.author = Some(kv[1].clone()),
                _ => eprintln!("{} Unknown key: {}", style("!").red(), kv[0]),
            }
            cfg.save()?;
            println!("  {} Set {} = {}", style("✓").green(), style(&kv[0]).cyan(), style(&kv[1]).yellow());
        }
        return Ok(());
    }

    if args.config_reset {
        config::Config::reset()?;
        println!("  {} Config reset to defaults.", style("✓").green());
        return Ok(());
    }

    let input = args.input.as_ref().expect("input_pdf is required");
    let input_path = Path::new(input);
    if !input_path.exists() {
        eprintln!("{} '{}' not found", style("Error:").red().bold(), input);
        std::process::exit(1);
    }

    // Load config and merge with CLI
    let cfg = config::Config::load();
    let cfg = cfg.merge_with_cli(
        args.template.clone(),
        args.title.clone(),
        args.author.clone(),
    );
    let fmt = args.format.clone().unwrap_or(cfg.format.clone());
    let template_name = cfg.template.clone();

    // Show banner
    print_banner();

    if input_path.is_dir() {
        run_batch(input_path, &args, &cfg, &fmt, &template_name)?;
    } else {
        run_single(input_path, &args, &cfg, &fmt, &template_name)?;
    }

    Ok(())
}

fn print_banner() {
    let logo = r#"
╔══════════════════════════════════════════╗
║        ██████╗  █████╗ ██████╗          ║
║        ██╔══██╗██╔══██╗██╔══██╗         ║
║        ██████╔╝███████║██████╔╝         ║
║        ██╔═══╝ ██╔══██║██╔═══╝          ║
║        ██║     ██║  ██║██║              ║
║        ╚═╝     ╚═╝  ╚═╝╚═╝              ║
║                                          ║
║  Stay productive. Read fiction.  v1.0.0  ║
╚══════════════════════════════════════════╝
"#;

    for line in logo.lines() {
        if line.is_empty() {
            println!();
        } else {
            println!("{}", style(line).color256(PURPLE));
        }
    }
}

fn print_templates() {
    println!("\n  {} Available Templates", style("✦").color256(PURPLE).bold());
    println!("  {}", style("──────────────────────").dim());
    for t in templates::list() {
        println!(
            "  {} {}  {}",
            style("▸").color256(INDIGO),
            style(t.name).bold(),
            style(t.description).dim()
        );
    }
    println!();
}

fn print_config() {
    let cfg = config::Config::load();
    println!("\n  {} PaperCoat Configuration", style("✦").color256(PURPLE).bold());
    println!("  {}", style("──────────────────────").dim());
    println!("  {} {}", style("template:").cyan(), style(&cfg.template).white());
    println!("  {} {}", style("format:").cyan(), style(&cfg.format).white());
    if let Some(t) = &cfg.title {
        println!("  {} {}", style("title:").cyan(), style(t).yellow());
    }
    if let Some(a) = &cfg.author {
        println!("  {} {}", style("author:").cyan(), style(a).yellow());
    }
    println!();
}

fn run_single(
    input: &Path,
    args: &cli::Args,
    cfg: &config::Config,
    fmt: &str,
    template: &str,
) -> anyhow::Result<()> {
    let mp = MultiProgress::new();
    let file_size = input.metadata().map(|m| m.len()).unwrap_or(0);

    // File info panel
    let file_info = format!(
        "{}  {}  {}  {}",
        style("┃").color256(PURPLE),
        style("File:").bold().white(),
        style(input.file_name().unwrap().to_string_lossy()).cyan(),
        style(format_size(file_size)).dim(),
    );
    println!("  {}", file_info);
    println!();

    // Step 1: Extract
    let extract_pb = mp.add(ProgressBar::new(100));
    extract_pb.set_style(
        ProgressStyle::with_template(
            &format!(
                "{{spinner:.{p}}} {{msg:.{p}}} {{bar:40.cyan/{p}}} {{percent}}%  ETA {{eta}}",
                p = PURPLE
            )
        )
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .progress_chars("━─"),
    );
    extract_pb.set_message("Extracting text");
    extract_pb.set_position(30);

    let extracted = extractor::extract_text(input, args.ocr)?;

    extract_pb.set_position(100);
    extract_pb.finish_with_message(format!(
        "{} {} words extracted",
        style("✓").green(),
        style(format_count(extracted.word_count)).cyan()
    ));

    // Stats mode
    if args.stats {
        mp.clear()?;
        let stats = transformer::compute_readability_stats(&extracted.text);
        println!("\n  {} Text Statistics", style("✦").color256(PURPLE).bold());
        println!("  {}", style("──────────────────────────────").dim());
        for (key, val) in &stats {
            println!("  {:30} {}", style(key).cyan(), style(val).yellow());
        }
        println!();
        return Ok(());
    }

    // Step 2: Transform
    let transform_pb = mp.add(ProgressBar::new(100));
    transform_pb.set_style(
        ProgressStyle::with_template(
            &format!(
                "{{spinner:.{p}}} {{msg:.{p}}} {{bar:40.cyan/{p}}} {{percent}}%  ETA {{eta}}",
                p = PURPLE
            )
        )
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .progress_chars("━─"),
    );
    transform_pb.set_message("Transforming into academic paper");
    transform_pb.set_position(10);

    let page_count = extractor::estimate_page_count(input);
    let paper = transformer::transform_text(
        &extracted.text,
        cfg.title.clone(),
        cfg.author.clone(),
        page_count,
    );

    transform_pb.set_position(100);
    transform_pb.finish_with_message(format!(
        "{} {} → {}",
        style("✓").green(),
        style(&paper.author_line).cyan(),
        style(&paper.title).yellow()
    ));

    // Step 3: Render output
    let render_pb = mp.add(ProgressBar::new(100));
    render_pb.set_style(
        ProgressStyle::with_template(
            &format!(
                "{{spinner:.{p}}} {{msg:.{p}}} {{bar:40.cyan/{p}}} {{percent}}%  ETA {{eta}}",
                p = PURPLE
            )
        )
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .progress_chars("━─"),
    );
    render_pb.set_position(10);

    let output_path = determine_output(input, args.output.as_deref(), fmt);
    let output_pb = output_path.clone();

    if fmt == "pdf" {
        // For PDF format, generate LaTeX and compile
        render_pb.set_message("Generating LaTeX");
        render_pb.set_position(30);
        let tex_path = output_pb.with_extension("tex");
        renderer::render_tex(&paper, &tex_path, template)?;

        render_pb.set_message("Compiling with pdflatex");
        render_pb.set_position(70);
        match renderer::compile_latex(&tex_path)? {
            Some(pdf_path) => {
                render_pb.set_position(100);
                render_pb.finish_with_message(format!(
                    "{} {}",
                    style("✓").green(),
                    style("PDF generated").cyan()
                ));
                print_results(&paper, &pdf_path, "pdf", fmt, &template);

                if args.open {
                    open_file(&pdf_path);
                }
            }
            None => {
                render_pb.finish_with_message(format!(
                    "{} LaTeX saved (pdflatex not found)",
                    style("!").yellow()
                ));
                print_results(&paper, &tex_path, "tex", fmt, &template);
            }
        }
    } else {
        render_pb.set_message("Generating LaTeX");
        renderer::render_tex(&paper, &output_path, template)?;

        render_pb.set_position(80);
        // Try compiling to PDF as bonus
        let _pdf_path = match renderer::compile_latex(&output_path)? {
            Some(p) => {
                render_pb.set_position(100);
                render_pb.finish_with_message(format!(
                    "{} LaTeX + PDF generated",
                    style("✓").green()
                ));
                if args.open {
                    open_file(&p);
                }
                Some(p)
            }
            None => {
                render_pb.set_position(100);
                render_pb.finish_with_message(format!(
                    "{} LaTeX saved",
                    style("✓").green()
                ));
                None
            }
        };

        print_results(&paper, &output_path, "tex", fmt, &template);
    }

    mp.clear()?;
    Ok(())
}

fn run_batch(
    dir: &Path,
    args: &cli::Args,
    cfg: &config::Config,
    fmt: &str,
    template: &str,
) -> anyhow::Result<()> {
    let pdfs: Vec<PathBuf> = glob_pdfs(dir);
    if pdfs.is_empty() {
        eprintln!("  {} No PDFs found in {}", style("!").yellow(), dir.display());
        return Ok(());
    }

    println!(
        "  {} {} {} {}",
        style("┃").color256(PURPLE),
        style("Batch:").bold().white(),
        style(pdfs.len()).cyan(),
        style("files found").dim()
    );
    println!();

    let mut success = 0;
    let total = pdfs.len();

    for (i, pdf) in pdfs.iter().enumerate() {
        let name = pdf.file_name().unwrap().to_string_lossy();
        print!(
            "  {} [{} {} {} {}] {} ",
            style("┃").color256(INDIGO),
            style(i + 1).cyan(),
            style("/").dim(),
            style(total).cyan(),
            style("]").dim(),
            style(name).white(),
        );

        match extractor::extract_text(pdf, args.ocr) {
            Ok(extracted) => {
                let paper = transformer::transform_text(
                    &extracted.text,
                    cfg.title.clone(),
                    cfg.author.clone(),
                    extractor::estimate_page_count(pdf),
                );
                let output_path = determine_output(pdf, args.output.as_deref(), fmt);
                match renderer::render_tex(&paper, &output_path, template) {
                    Ok(_) => {
                        println!("{}", style("✓").green());
                        success += 1;
                    }
                    Err(e) => {
                        println!("{} {}", style("✗").red(), e);
                    }
                }
            }
            Err(e) => {
                println!("{} {}", style("✗").red(), e);
            }
        }
    }

    println!(
        "\n  {} {} {} {} {}",
        style("┃").color256(PURPLE),
        style("Done:").bold().white(),
        style(success).green(),
        style("/").dim(),
        style(total).cyan(),
    );

    Ok(())
}

fn determine_output(input: &Path, output_override: Option<&str>, fmt: &str) -> PathBuf {
    if let Some(out) = output_override {
        PathBuf::from(out)
    } else {
        let ext = if fmt == "pdf" { "pdf" } else { "tex" };
        let stem = input.file_stem().unwrap().to_string_lossy();
        input.with_file_name(format!("{}_academic.{}", stem, ext))
    }
}

fn print_results(paper: &transformer::PaperData, output: &Path, output_fmt: &str, requested_fmt: &str, template: &str) {
    let size = output.metadata().map(|m| m.len()).unwrap_or(0);

    println!();
    println!("  {} Results", style("✦").color256(PURPLE).bold());
    println!("  {}", style("──────────────────────────────────────────────").dim());

    let name = output.file_name().unwrap().to_string_lossy();
    println!("  {} {}", style("Output:").cyan(), style(name).green().bold());

    let fmt_display = if output_fmt == "pdf" {
        format!("PDF ({}p)", paper.page_count)
    } else if requested_fmt == "pdf" && output_fmt == "tex" {
        "LaTeX (pdflatex not found)".to_string()
    } else {
        "LaTeX".to_string()
    };

    println!("  {} {}", style("Format:").cyan(), style(fmt_display).white());
    println!("  {} {}", style("Size:").cyan(), style(format_size(size)).white());
    println!("  {} {}", style("Words:").cyan(), style(format_count(paper.word_count)).white());
    println!("  {} {}", style("Title:").cyan(), style(&paper.title).yellow());
    println!("  {} {}", style("Author:").cyan(), style(&paper.author_line).white());
    println!("  {} {}", style("Template:").cyan(), style(template).white());
    println!("  {} {}", style("Journal:").cyan(), style(&paper.journal).magenta());
    println!("  {} {}", style("DOI:").cyan(), style(&paper.doi).dim());
    println!("  {} {} {} {} {} {}", style("Vol:").cyan(), style(&paper.volume).white(), style("Iss:").cyan(), style(&paper.issue).white(), style("Pages:").cyan(), style(paper.page_count).white());
    println!("  {} {}", style("Keywords:").cyan(), style(paper.keywords.join("; ")).dim());
    println!("  {} {}", style("References:").cyan(), style(paper.references.len()).white());
    println!();
}

fn open_file(path: &Path) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else {
        "open"
    };
    let _ = std::process::Command::new(opener).arg(path).output();
}

fn glob_pdfs(dir: &Path) -> Vec<PathBuf> {
    let mut pdfs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "pdf").unwrap_or(false) {
                pdfs.push(path);
            }
        }
    }
    pdfs.sort();
    pdfs
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 * 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn format_count(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}
