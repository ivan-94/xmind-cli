use std::path::PathBuf;

use clap::{Args, Subcommand};
use clap_complete::Shell;

use super::args::MarkdownMode;

const INSPECT_EXAMPLES: &str = r#"Examples:
  xmind inspect roadmap.xmind
  xmind inspect roadmap.xmind --json"#;
const SHEETS_EXAMPLES: &str = r#"Examples:
  xmind sheets roadmap.xmind
  xmind sheets roadmap.xmind --json --fields title,id"#;
const TREE_EXAMPLES: &str = r#"Examples:
  xmind tree roadmap.xmind --depth 2
  xmind tree roadmap.xmind --sheet "Roadmap" --include-assets"#;
const FIND_EXAMPLES: &str = r#"Examples:
  xmind find roadmap.xmind --title "Launch"
  xmind find roadmap.xmind --query 'title contains "Launch"' --limit 5"#;
const GET_EXAMPLES: &str = r#"Examples:
  xmind get roadmap.xmind --node path:/Launch
  xmind get roadmap.xmind --node topic-1 --depth 2 --include-assets"#;
const ADD_EXAMPLES: &str = r#"Examples:
  xmind add roadmap.xmind --parent path:/Launch --title "Risks" --dry-run
  xmind add roadmap.xmind --parent path:/Launch --title "Risks" --apply --backup"#;
const ADD_TREE_EXAMPLES: &str = r#"Examples:
  xmind add-tree roadmap.xmind --parent path:/Launch --input tree.yaml --dry-run
  xmind add-tree roadmap.xmind --parent path:/Launch --from-markdown notes.md --markdown-mode auto --dry-run"#;
const SET_EXAMPLES: &str = r#"Examples:
  xmind set roadmap.xmind --node path:/Launch --title "Launch plan" --dry-run
  xmind set roadmap.xmind --node path:/Launch --append-note "Owner: PM" --apply --backup
  xmind set roadmap.xmind --node path:/Launch --add-label priority --add-marker task-start --dry-run"#;
const DELETE_EXAMPLES: &str = r#"Examples:
  xmind delete roadmap.xmind --node path:/Launch/Risks --dry-run
  xmind delete roadmap.xmind --node path:/Launch --children-only --apply --backup"#;
const MOVE_EXAMPLES: &str = r#"Examples:
  xmind move roadmap.xmind --node path:/Risks --to path:/Launch --dry-run
  xmind move roadmap.xmind --node path:/Risks --to path:/Launch --position first --apply --backup"#;
const COPY_EXAMPLES: &str = r#"Examples:
  xmind copy roadmap.xmind --node path:/Launch --to path:/Archive --dry-run
  xmind copy roadmap.xmind --node path:/Launch --to path:/Archive --title "Launch copy" --apply --backup"#;
const PATCH_EXAMPLES: &str = r#"Examples:
  xmind patch roadmap.xmind --ops patch.yaml --dry-run
  xmind patch roadmap.xmind --ops patch.json --apply --backup
  xmind patch roadmap.xmind --ops docs/examples/patch-add-tree.yaml --dry-run --json"#;
const DIFF_EXAMPLES: &str = r#"Examples:
  xmind diff roadmap.xmind
  xmind diff roadmap.xmind --json"#;
const VALIDATE_EXAMPLES: &str = r#"Examples:
  xmind validate roadmap.xmind
  xmind validate roadmap.xmind --strict --json"#;
const EXPORT_EXAMPLES: &str = r#"Examples:
  xmind export roadmap.xmind --format markdown --output roadmap.md
  xmind export roadmap.xmind --format assets --output exported-assets --overwrite"#;
const IMPORT_EXAMPLES: &str = r#"Examples:
  xmind import --input roadmap.md --output roadmap.xmind --dry-run
  xmind import --input roadmap.md --into existing.xmind --parent path:/Launch --dry-run
  xmind import --input roadmap.md --into existing.xmind --parent path:/Launch --apply --backup"#;
const BACKUP_EXAMPLES: &str = r#"Examples:
  xmind backup roadmap.xmind
  xmind backup roadmap.xmind --backup-dir ./.backups"#;
const RESTORE_EXAMPLES: &str = r#"Examples:
  xmind restore roadmap.xmind --dry-run
  xmind restore roadmap.xmind --apply --backup"#;
const COMPLETION_EXAMPLES: &str = r#"Examples:
  xmind completion zsh > ~/.zfunc/_xmind
  xmind completion bash > xmind.bash"#;

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Summarize workbook metadata, sheets, and compatibility details.",
        after_help = INSPECT_EXAMPLES
    )]
    Inspect(WorkbookCommand),
    #[command(
        about = "List sheets in a workbook with ids, titles, and ordering.",
        after_help = SHEETS_EXAMPLES
    )]
    Sheets(WorkbookCommand),
    #[command(
        about = "Print a workbook topic tree for quick reading or scripting.",
        after_help = TREE_EXAMPLES
    )]
    Tree(TreeCommand),
    #[command(
        about = "Search workbook topics by title, text, or query selector.",
        after_help = FIND_EXAMPLES
    )]
    Find(FindCommand),
    #[command(
        about = "Read one topic and optionally include its descendants.",
        after_help = GET_EXAMPLES
    )]
    Get(GetCommand),
    #[command(
        about = "Add a single topic under an existing parent topic.",
        after_help = ADD_EXAMPLES
    )]
    Add(AddCommand),
    #[command(
        about = "Add a topic subtree from YAML, JSON, or Markdown input.",
        after_help = ADD_TREE_EXAMPLES
    )]
    AddTree(AddTreeCommand),
    #[command(
        about = "Update topic fields such as title, notes, labels, markers, links, or images.",
        after_help = SET_EXAMPLES
    )]
    Set(Box<SetCommand>),
    #[command(
        about = "Delete a topic or selected descendants from a workbook.",
        after_help = DELETE_EXAMPLES
    )]
    Delete(DeleteCommand),
    #[command(
        about = "Move a topic to a different parent or sibling position.",
        after_help = MOVE_EXAMPLES
    )]
    Move(MoveCommand),
    #[command(
        about = "Copy a topic subtree to another location in the workbook.",
        after_help = COPY_EXAMPLES
    )]
    Copy(CopyCommand),
    #[command(
        about = "Apply a batch of structured workbook edit operations.",
        after_help = PATCH_EXAMPLES
    )]
    Patch(PatchCommand),
    #[command(
        about = "Show structural differences for the current workbook diff surface.",
        after_help = DIFF_EXAMPLES
    )]
    Diff(WorkbookCommand),
    #[command(
        about = "Check whether a workbook can be read and safely processed.",
        after_help = VALIDATE_EXAMPLES
    )]
    Validate(ValidateCommand),
    #[command(
        about = "Export workbook content to Markdown, outline, JSON, or assets.",
        after_help = EXPORT_EXAMPLES
    )]
    Export(ExportCommand),
    #[command(
        about = "Create or update a workbook from Markdown or structured input.",
        after_help = IMPORT_EXAMPLES
    )]
    Import(ImportCommand),
    #[command(
        about = "Create an explicit backup copy of a workbook.",
        after_help = BACKUP_EXAMPLES
    )]
    Backup(BackupCommand),
    #[command(
        about = "Restore workbook content from the matching backup workflow.",
        after_help = RESTORE_EXAMPLES
    )]
    Restore(MutationCommand),
    #[command(
        about = "Generate shell completion scripts for the xmind command.",
        after_help = COMPLETION_EXAMPLES
    )]
    Completion(CompletionCommand),
}

#[derive(Debug, Args)]
pub struct WorkbookCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,
}

#[derive(Debug, Args)]
pub struct TreeCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Limit how many topic levels to print")]
    pub depth: Option<usize>,

    #[arg(long, help = "Include image and attachment references when available")]
    pub include_assets: bool,
}

#[derive(Debug, Args)]
pub struct FindCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Match an exact topic title")]
    pub title: Option<String>,

    #[arg(long, help = "Match topics whose title contains this text")]
    pub title_contains: Option<String>,

    #[arg(
        long,
        help = "Match topics whose searchable content contains this text"
    )]
    pub contains: Option<String>,

    #[arg(long, help = "Run a query selector expression")]
    pub query: Option<String>,

    #[arg(long, help = "Maximum number of matches to return")]
    pub limit: Option<usize>,

    #[arg(long, help = "Skip this many matches before returning results")]
    pub offset: Option<usize>,
}

#[derive(Debug, Args)]
pub struct GetCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Topic selector such as an id or path:/Title")]
    pub node: String,

    #[arg(long, help = "Limit descendant depth included in the result")]
    pub depth: Option<usize>,

    #[arg(long, help = "Include image and attachment references when available")]
    pub include_assets: bool,
}

#[derive(Debug, Args)]
pub struct ValidateCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Treat warnings as validation failures")]
    pub strict: bool,
}

#[derive(Debug, Args)]
pub struct BackupCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Directory for the backup copy")]
    pub backup_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ExportCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Write exported content to a file or directory")]
    pub output: Option<PathBuf>,

    #[arg(long, help = "Allow replacing an existing output path")]
    pub overwrite: bool,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false, args = ["output", "into"])]
pub struct ImportCommand {
    #[arg(long, help = "Markdown, YAML, or JSON input to import")]
    pub input: PathBuf,

    #[arg(long, help = "Create a new workbook at this path")]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        value_name = "WORKBOOK",
        help = "Import into an existing workbook"
    )]
    pub into: Option<PathBuf>,

    #[arg(
        long,
        help = "Parent selector when importing into an existing workbook"
    )]
    pub parent: Option<String>,

    #[arg(long, help = "Allow replacing an existing output workbook")]
    pub overwrite: bool,

    #[arg(long, help = "Create a safety backup before applying into a workbook")]
    pub backup: bool,

    #[arg(long, value_enum, help = "Markdown outline parsing mode")]
    pub markdown_mode: Option<MarkdownMode>,

    #[command(flatten)]
    pub mode: ApplyMode,
}

#[derive(Debug, Args)]
pub struct MutationMode {
    #[command(flatten)]
    pub apply_mode: ApplyMode,

    #[arg(long, help = "Create a safety backup before applying changes")]
    pub backup: bool,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct ApplyMode {
    #[arg(long, help = "Preview the change without writing the workbook")]
    pub dry_run: bool,

    #[arg(long, help = "Write the change to the workbook")]
    pub apply: bool,
}

#[derive(Debug, Args)]
pub struct MutationCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct AddCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Parent topic selector")]
    pub parent: String,

    #[arg(long, help = "Title for the new topic")]
    pub title: String,

    #[arg(
        long,
        help = "Insertion position such as first, last, before:<selector>, or after:<selector>"
    )]
    pub position: Option<String>,

    #[arg(long, help = "Create missing path segments for path selectors")]
    pub create_missing_path: bool,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct AddTreeCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Parent topic selector")]
    pub parent: String,

    #[arg(long, help = "YAML or JSON tree input file")]
    pub input: Option<PathBuf>,

    #[arg(long, help = "Markdown outline input file")]
    pub from_markdown: Option<PathBuf>,

    #[arg(long, value_enum, help = "Markdown outline parsing mode")]
    pub markdown_mode: Option<MarkdownMode>,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct SetCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Topic selector to update")]
    pub node: String,

    #[arg(long, help = "Replace the topic title")]
    pub title: Option<String>,

    #[arg(long, help = "Replace the topic note")]
    pub note: Option<String>,

    #[arg(long, help = "Append text to the topic note")]
    pub append_note: Option<String>,

    #[arg(long, help = "Replace all labels with a comma-separated list")]
    pub set_labels: Option<String>,

    #[arg(long, help = "Add one label")]
    pub add_label: Option<String>,

    #[arg(long, help = "Remove one label")]
    pub remove_label: Option<String>,

    #[arg(long, help = "Replace all markers with a comma-separated list")]
    pub set_markers: Option<String>,

    #[arg(long, help = "Add one marker id")]
    pub add_marker: Option<String>,

    #[arg(long, help = "Remove one marker id")]
    pub remove_marker: Option<String>,

    #[arg(long, help = "Set the topic hyperlink")]
    pub hyperlink: Option<String>,

    #[arg(long, help = "Attach an image from this path")]
    pub image: Option<PathBuf>,

    #[arg(long, help = "Alt text for the attached image")]
    pub image_alt: Option<String>,

    #[arg(long, help = "Title text for the attached image")]
    pub image_title: Option<String>,

    #[arg(
        long,
        value_name = "FIELD",
        help = "Clear a field such as note or hyperlink"
    )]
    pub clear: Vec<String>,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct DeleteCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Topic selector to delete")]
    pub node: String,

    #[arg(long, help = "Delete only the selected topic's children")]
    pub children_only: bool,

    #[arg(long, help = "Move children to the deleted topic's parent")]
    pub promote_children: bool,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct MoveCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Topic selector to move")]
    pub node: String,

    #[arg(long, help = "Destination parent topic selector")]
    pub to: String,

    #[arg(long, help = "Insertion position at the destination")]
    pub position: Option<String>,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct CopyCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "Topic selector to copy")]
    pub node: String,

    #[arg(long, help = "Destination parent topic selector")]
    pub to: String,

    #[arg(long, help = "Optional title override for the copied root topic")]
    pub title: Option<String>,

    #[arg(long, help = "Insertion position at the destination")]
    pub position: Option<String>,

    #[arg(long, help = "Preserve topic ids for diagnostic flows")]
    pub preserve_ids: bool,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct PatchCommand {
    #[arg(help = "Path to the .xmind workbook")]
    pub workbook: PathBuf,

    #[arg(long, help = "YAML or JSON file containing patch operations")]
    pub ops: PathBuf,

    #[command(flatten)]
    pub mode: MutationMode,
}

#[derive(Debug, Args)]
pub struct CompletionCommand {
    #[arg(help = "Shell to generate completions for")]
    pub shell: Shell,
}
