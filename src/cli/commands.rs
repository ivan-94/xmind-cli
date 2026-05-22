use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::args::MarkdownMode;

#[derive(Debug, Subcommand)]
pub enum Command {
    Inspect(WorkbookCommand),
    Sheets(WorkbookCommand),
    Tree(TreeCommand),
    Find(FindCommand),
    Get(GetCommand),
    Add(AddCommand),
    AddTree(AddTreeCommand),
    Set(SetCommand),
    Delete(DeleteCommand),
    Move(MoveCommand),
    Copy(CopyCommand),
    Patch(PatchCommand),
    Diff(WorkbookCommand),
    Validate(ValidateCommand),
    Export(WorkbookCommand),
    Import(MutationCommand),
    Backup(BackupCommand),
    Restore(MutationCommand),
}

#[derive(Debug, Args)]
pub struct WorkbookCommand {
    pub workbook: PathBuf,
}

#[derive(Debug, Args)]
pub struct TreeCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub depth: Option<usize>,

    #[arg(long)]
    pub include_assets: bool,
}

#[derive(Debug, Args)]
pub struct FindCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long)]
    pub title_contains: Option<String>,

    #[arg(long)]
    pub contains: Option<String>,

    #[arg(long)]
    pub query: Option<String>,

    #[arg(long)]
    pub limit: Option<usize>,

    #[arg(long)]
    pub offset: Option<usize>,
}

#[derive(Debug, Args)]
pub struct GetCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub node: String,

    #[arg(long)]
    pub depth: Option<usize>,

    #[arg(long)]
    pub include_assets: bool,
}

#[derive(Debug, Args)]
pub struct ValidateCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub strict: bool,
}

#[derive(Debug, Args)]
pub struct BackupCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub backup_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct MutationMode {
    #[command(flatten)]
    pub apply_mode: ApplyMode,

    #[arg(long)]
    pub backup: bool,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct ApplyMode {
    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub apply: bool,
}

#[derive(Debug, Args)]
pub struct MutationCommand {
    pub workbook: PathBuf,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct AddCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub parent: String,

    #[arg(long)]
    pub title: String,

    #[arg(long)]
    pub position: Option<String>,

    #[arg(long)]
    pub create_missing_path: bool,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct AddTreeCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub parent: String,

    #[arg(long)]
    pub input: Option<PathBuf>,

    #[arg(long)]
    pub from_markdown: Option<PathBuf>,

    #[arg(long, value_enum)]
    pub markdown_mode: Option<MarkdownMode>,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct SetCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub node: String,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long)]
    pub note: Option<String>,

    #[arg(long)]
    pub append_note: Option<String>,

    #[arg(long)]
    pub set_labels: Option<String>,

    #[arg(long)]
    pub add_label: Option<String>,

    #[arg(long)]
    pub remove_label: Option<String>,

    #[arg(long)]
    pub set_markers: Option<String>,

    #[arg(long)]
    pub add_marker: Option<String>,

    #[arg(long)]
    pub remove_marker: Option<String>,

    #[arg(long)]
    pub hyperlink: Option<String>,

    #[arg(long, value_name = "FIELD")]
    pub clear: Vec<String>,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct DeleteCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub node: String,

    #[arg(long)]
    pub children_only: bool,

    #[arg(long)]
    pub promote_children: bool,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct MoveCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub node: String,

    #[arg(long)]
    pub to: String,

    #[arg(long)]
    pub position: Option<String>,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct CopyCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub node: String,

    #[arg(long)]
    pub to: String,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long)]
    pub position: Option<String>,

    #[arg(long)]
    pub preserve_ids: bool,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct PatchCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub ops: PathBuf,

    #[command(flatten)]
    pub mode: MutationMode,
}
