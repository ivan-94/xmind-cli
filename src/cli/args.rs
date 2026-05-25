use clap::{Parser, ValueEnum};

use super::commands::Command;

const TOP_LEVEL_EXAMPLES: &str = r#"Examples:
  xmind tree roadmap.xmind --depth 2
  xmind find roadmap.xmind --title "Launch"
  xmind set roadmap.xmind --node path:/Launch --title "Launch plan" --dry-run
  xmind export roadmap.xmind --format markdown --output roadmap.md"#;

#[derive(Debug, Parser)]
#[command(
    name = "xmind",
    version,
    about = "Inspect and edit XMind workbooks through an agent-native CLI",
    after_help = TOP_LEVEL_EXAMPLES
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, global = true, help = "Emit a JSON command envelope")]
    pub json: bool,

    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = OutputFormat::Text,
        help = "Select the human or machine-readable output format"
    )]
    pub format: OutputFormat,

    #[arg(
        long,
        global = true,
        value_delimiter = ',',
        help = "Limit JSON output to comma-separated field paths"
    )]
    pub fields: Vec<String>,

    #[arg(long, global = true, help = "Suppress non-essential text output")]
    pub quiet: bool,

    #[arg(long, global = true, help = "Disable ANSI color in text output")]
    pub no_color: bool,

    #[arg(long, global = true, help = "Select a sheet by title")]
    pub sheet: Option<String>,

    #[arg(long, global = true, help = "Select a sheet by stable id")]
    pub sheet_id: Option<String>,

    #[arg(long, global = true, help = "Select a sheet by zero-based index")]
    pub sheet_index: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    CompactJson,
    Markdown,
    Outline,
    Assets,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum MarkdownMode {
    Heading,
    List,
    Hybrid,
    Auto,
}
