use clap::{Parser, ValueEnum};

use super::commands::Command;

#[derive(Debug, Parser)]
#[command(
    name = "xmind",
    version,
    about = "Inspect and edit XMind workbooks through an agent-native CLI"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, global = true)]
    pub json: bool,

    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    #[arg(long, global = true, value_delimiter = ',')]
    pub fields: Vec<String>,

    #[arg(long, global = true)]
    pub quiet: bool,

    #[arg(long, global = true)]
    pub no_color: bool,

    #[arg(long, global = true)]
    pub sheet: Option<String>,

    #[arg(long, global = true)]
    pub sheet_id: Option<String>,

    #[arg(long, global = true)]
    pub sheet_index: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    CompactJson,
    Markdown,
    Outline,
}
