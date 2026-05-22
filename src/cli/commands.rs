use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Command {
    Inspect(WorkbookCommand),
    Sheets(WorkbookCommand),
    Tree(TreeCommand),
    Find(FindCommand),
    Get(GetCommand),
    Add(MutationCommand),
    AddTree(MutationCommand),
    Set(MutationCommand),
    Delete(MutationCommand),
    Move(MutationCommand),
    Copy(MutationCommand),
    Patch(PatchCommand),
    Diff(WorkbookCommand),
    Validate(ValidateCommand),
    Export(WorkbookCommand),
    Import(MutationCommand),
    Backup(WorkbookCommand),
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
}

#[derive(Debug, Args)]
pub struct ValidateCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub strict: bool,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct MutationMode {
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
pub struct PatchCommand {
    pub workbook: PathBuf,

    #[arg(long)]
    pub ops: PathBuf,

    #[command(flatten)]
    pub mode: MutationMode,
}
