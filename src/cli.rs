use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "papercoat",
    version,
    about = "Turn any novel PDF into a convincing academic paper",
    after_help = "📚  Stay productive. Read fiction.",
    styles = papercoat_styles()
)]
pub struct Args {
    /// Input PDF file or directory (for batch mode)
    #[arg(required = false)]
    pub input: Option<String>,

    /// Output path
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Journal template
    #[arg(short = 't', long, default_value = None)]
    pub template: Option<String>,

    /// Output format (tex or pdf)
    #[arg(short = 'f', long, default_value = None)]
    pub format: Option<String>,

    /// Override auto-generated title
    #[arg(long)]
    pub title: Option<String>,

    /// Override author name
    #[arg(long)]
    pub author: Option<String>,

    /// Use OCR for scanned PDFs
    #[arg(long)]
    pub ocr: bool,

    /// Open the result after generation
    #[arg(long)]
    pub open: bool,

    /// Show text statistics instead of transforming
    #[arg(long)]
    pub stats: bool,

    /// List available templates and exit
    #[arg(long)]
    pub list_templates: bool,

    /// Show current configuration
    #[arg(long)]
    pub show_config: bool,

    /// Set a config value (KEY VALUE)
    #[arg(long, num_args = 2)]
    pub config_set: Option<Vec<String>>,

    /// Reset config to defaults
    #[arg(long)]
    pub config_reset: bool,
}

fn papercoat_styles() -> clap::builder::Styles {
    use clap::builder::styling::{AnsiColor, Effects, Style};
    clap::builder::Styles::styled()
        .header(
            Style::new()
                .bold()
                .fg_color(Some(AnsiColor::BrightMagenta.into())),
        )
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(AnsiColor::BrightMagenta.into())),
        )
        .literal(Style::new().fg_color(Some(AnsiColor::Cyan.into())))
        .placeholder(Style::new().fg_color(Some(AnsiColor::BrightCyan.into())))
        .error(
            Style::new()
                .fg_color(Some(AnsiColor::Red.into()))
                .effects(Effects::BOLD),
        )
        .valid(Style::new().fg_color(Some(AnsiColor::Green.into())))
        .invalid(Style::new().fg_color(Some(AnsiColor::Yellow.into())))
}
