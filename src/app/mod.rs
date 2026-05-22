mod patch;
mod tree_input;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use crate::cli::{
    CandidateDto, Cli, CliErrorBody, Command, CommandEnvelope, ErrorCode, MarkdownMode,
    OutputFormat,
};
use crate::domain::diff::{Diff, DiffEvent, FieldChange};
use crate::domain::mutation::{AddTopicRequest, MutationPlanner};
use crate::domain::path::TopicPath;
use crate::domain::query::QueryExpr;
use crate::domain::selector::Selector;
use crate::domain::sheet::Sheet;
use crate::domain::topic::Topic;
use crate::infra::fs::backup::{create_backup, create_backup_in_dir, BackupError};
use crate::infra::xmind::encode::{InsertPosition, TopicClearField, XMindWriteError};
use crate::render::diff::render_human_outline;

use self::patch::render_patch;
use self::tree_input::{
    read_tree_input, validate_topic_tree_input, TopicTreeImageInputDto, TopicTreeInputDto,
};

pub fn run(cli: Cli) -> i32 {
    let Cli {
        command,
        json,
        format,
        fields,
        quiet,
        no_color,
        sheet,
        sheet_id,
        sheet_index,
    } = cli;

    let compact_json = format == OutputFormat::CompactJson;
    let sheet_selection = SheetSelection {
        title: sheet,
        id: sheet_id,
        index: sheet_index,
    };

    let Some(invocation) = Invocation::from_command(command, sheet_selection, quiet)
        .map(|invocation| invocation.with_no_color(no_color))
    else {
        return 0;
    };

    if let Err(error) = validate_fields(&invocation.command, &fields) {
        return render_error(invocation, json, error);
    }

    if !invocation.workbook.exists() {
        let path = invocation.workbook.display().to_string();
        let error = CliErrorBody::new(
            ErrorCode::FileNotFound,
            format!("Workbook not found: {path}"),
            true,
            "Check the workbook path and retry.",
        )
        .with_path(path);

        return render_error(invocation, json, error);
    }

    match invocation.action {
        Action::Inspect => render_inspect(invocation, json, &fields, compact_json),
        Action::Sheets => render_sheets(invocation, json, &fields, compact_json),
        Action::Tree { depth } => render_tree(invocation, json, depth, &fields, compact_json),
        Action::Find {
            ref title,
            ref title_contains,
            ref contains,
            ref query,
            limit,
            offset,
        } => {
            let title = title.clone();
            let title_contains = title_contains.clone();
            let contains = contains.clone();
            let query = query.clone();
            render_find(
                invocation,
                json,
                FindRenderOptions {
                    title,
                    title_contains,
                    contains,
                    query,
                    limit,
                    offset,
                    fields: fields.clone(),
                    compact_json,
                },
            )
        }
        Action::Get { ref node, depth } => {
            let node = node.clone();
            render_get(invocation, json, &node, depth, &fields, compact_json)
        }
        Action::Validate { strict } => render_validate(invocation, json, strict),
        Action::Backup { ref backup_dir } => {
            let backup_dir = backup_dir.clone();
            render_backup(invocation, json, backup_dir)
        }
        Action::Patch { ref ops } => {
            let ops = ops.clone();
            render_patch(invocation, json, &ops)
        }
        Action::Add {
            ref parent,
            ref title,
            ref position,
            create_missing_path,
        } => {
            let parent = parent.clone();
            let title = title.clone();
            let position = position.clone();
            render_add(
                invocation,
                json,
                &parent,
                &title,
                position,
                create_missing_path,
            )
        }
        Action::AddTree {
            ref parent,
            ref input,
            ref from_markdown,
            ref markdown_mode,
        } => {
            let parent = parent.clone();
            let input = input.clone();
            let from_markdown = from_markdown.clone();
            let markdown_mode = *markdown_mode;
            render_add_tree(
                invocation,
                json,
                &parent,
                input.as_deref(),
                from_markdown.as_deref(),
                markdown_mode,
            )
        }
        Action::SetTitle {
            ref node,
            ref title,
        } => {
            let node = node.clone();
            let title = title.clone();
            render_set_title(invocation, json, &node, &title)
        }
        Action::SetNote { ref node, ref note } => {
            let node = node.clone();
            let note = note.clone();
            render_set_note(invocation, json, &node, &note)
        }
        Action::SetAppendNote {
            ref node,
            ref append_note,
        } => {
            let node = node.clone();
            let append_note = append_note.clone();
            render_set_append_note(invocation, json, &node, &append_note)
        }
        Action::SetLabels {
            ref node,
            ref labels,
        } => {
            let node = node.clone();
            let labels = labels.clone();
            render_set_labels(invocation, json, &node, labels)
        }
        Action::SetAddLabel {
            ref node,
            ref label,
        } => {
            let node = node.clone();
            let label = label.clone();
            render_set_add_label(invocation, json, &node, label)
        }
        Action::SetRemoveLabel {
            ref node,
            ref label,
        } => {
            let node = node.clone();
            let label = label.clone();
            render_set_remove_label(invocation, json, &node, label)
        }
        Action::SetMarkers {
            ref node,
            ref markers,
        } => {
            let node = node.clone();
            let markers = markers.clone();
            render_set_markers(invocation, json, &node, markers)
        }
        Action::SetAddMarker {
            ref node,
            ref marker,
        } => {
            let node = node.clone();
            let marker = marker.clone();
            render_set_add_marker(invocation, json, &node, marker)
        }
        Action::SetRemoveMarker {
            ref node,
            ref marker,
        } => {
            let node = node.clone();
            let marker = marker.clone();
            render_set_remove_marker(invocation, json, &node, marker)
        }
        Action::SetHyperlink {
            ref node,
            ref hyperlink,
        } => {
            let node = node.clone();
            let hyperlink = hyperlink.clone();
            render_set_hyperlink(invocation, json, &node, &hyperlink)
        }
        Action::SetClear {
            ref node,
            ref fields,
        } => {
            let node = node.clone();
            let fields = fields.clone();
            render_set_clear(invocation, json, &node, fields)
        }
        Action::Delete {
            ref node,
            children_only,
            promote_children,
        } => {
            let node = node.clone();
            render_delete(invocation, json, &node, children_only, promote_children)
        }
        Action::Move {
            ref node,
            ref destination,
            ref position,
        } => {
            let node = node.clone();
            let destination = destination.clone();
            let position = position.clone();
            render_move(invocation, json, &node, &destination, position)
        }
        Action::Copy {
            ref node,
            ref destination,
            ref title,
            ref position,
            preserve_ids,
        } => {
            let node = node.clone();
            let destination = destination.clone();
            let title = title.clone();
            let position = position.clone();
            render_copy(
                invocation,
                json,
                &node,
                &destination,
                title,
                position,
                preserve_ids,
            )
        }
        Action::Noop => 0,
    }
}

#[derive(Clone)]
pub(super) struct Invocation {
    command: String,
    workbook: std::path::PathBuf,
    dry_run: bool,
    applied: bool,
    backup: bool,
    quiet: bool,
    no_color: bool,
    sheet_selection: SheetSelection,
    action: Action,
}

#[derive(Clone)]
struct SheetSelection {
    title: Option<String>,
    id: Option<String>,
    index: Option<usize>,
}

struct FindInvocationArgs {
    title: Option<String>,
    title_contains: Option<String>,
    contains: Option<String>,
    query: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Clone)]
enum Action {
    Inspect,
    Sheets,
    Tree {
        depth: Option<usize>,
    },
    Find {
        title: Option<String>,
        title_contains: Option<String>,
        contains: Option<String>,
        query: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
    },
    Get {
        node: String,
        depth: Option<usize>,
    },
    Patch {
        ops: std::path::PathBuf,
    },
    Add {
        parent: String,
        title: String,
        position: Option<String>,
        create_missing_path: bool,
    },
    AddTree {
        parent: String,
        input: Option<std::path::PathBuf>,
        from_markdown: Option<std::path::PathBuf>,
        markdown_mode: Option<MarkdownMode>,
    },
    SetTitle {
        node: String,
        title: String,
    },
    SetNote {
        node: String,
        note: String,
    },
    SetAppendNote {
        node: String,
        append_note: String,
    },
    SetLabels {
        node: String,
        labels: Vec<String>,
    },
    SetAddLabel {
        node: String,
        label: String,
    },
    SetRemoveLabel {
        node: String,
        label: String,
    },
    SetMarkers {
        node: String,
        markers: Vec<String>,
    },
    SetAddMarker {
        node: String,
        marker: String,
    },
    SetRemoveMarker {
        node: String,
        marker: String,
    },
    SetHyperlink {
        node: String,
        hyperlink: String,
    },
    SetClear {
        node: String,
        fields: Vec<String>,
    },
    Delete {
        node: String,
        children_only: bool,
        promote_children: bool,
    },
    Move {
        node: String,
        destination: String,
        position: Option<String>,
    },
    Copy {
        node: String,
        destination: String,
        title: Option<String>,
        position: Option<String>,
        preserve_ids: bool,
    },
    Validate {
        strict: bool,
    },
    Backup {
        backup_dir: Option<std::path::PathBuf>,
    },
    Noop,
}

fn count_topics(topic: &Topic) -> usize {
    1 + topic.children.iter().map(count_topics).sum::<usize>()
}

fn validate_fields(command: &str, fields: &[String]) -> Result<(), CliErrorBody> {
    for field in fields {
        if !allowed_fields(command).contains(&field.as_str()) {
            return Err(CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Unknown field for {command}: {field}"),
                true,
                "Use a field documented for this command, or omit --fields.",
            )
            .with_field_path("fields")
            .with_details(serde_json::json!({ "field": field })));
        }
    }

    Ok(())
}

fn allowed_fields(command: &str) -> &'static [&'static str] {
    match command {
        "inspect" => &[
            "file",
            "format",
            "sheet_count",
            "sheets",
            "resources_count",
            "capabilities",
        ],
        "sheets" => &["id", "index", "title", "root_topic_id", "topic_count"],
        "tree" => &[
            "id",
            "path",
            "title",
            "children_count",
            "children",
            "note",
            "labels",
            "markers",
            "hyperlink",
            "image",
            "depth",
        ],
        "get" => &[
            "id",
            "path",
            "title",
            "note",
            "labels",
            "markers",
            "hyperlink",
            "image",
            "children_count",
            "children",
            "depth",
        ],
        "find" => &["id", "path", "title", "sheet", "children_count"],
        _ => &[],
    }
}

impl Invocation {
    fn from_command(
        command: Option<Command>,
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Option<Self> {
        match command? {
            Command::Inspect(command) => {
                Some(Self::inspect(command.workbook, sheet_selection, quiet))
            }
            Command::Sheets(command) => {
                Some(Self::sheets(command.workbook, sheet_selection, quiet))
            }
            Command::Find(command) => Some(Self::find(
                command.workbook,
                FindInvocationArgs {
                    title: command.title,
                    title_contains: command.title_contains,
                    contains: command.contains,
                    query: command.query,
                    limit: command.limit,
                    offset: command.offset,
                },
                sheet_selection,
                quiet,
            )),
            Command::Get(command) => Some(Self::get(
                command.workbook,
                command.node,
                command.depth,
                command.include_assets,
                sheet_selection,
                quiet,
            )),
            Command::Diff(command) => Some(Self::read("diff", command.workbook, sheet_selection)),
            Command::Validate(command) => Some(Self::validate(
                command.workbook,
                command.strict,
                sheet_selection,
                quiet,
            )),
            Command::Export(command) => {
                Some(Self::read("export", command.workbook, sheet_selection))
            }
            Command::Backup(command) => Some(
                Self::read("backup", command.workbook, sheet_selection).with_action(
                    Action::Backup {
                        backup_dir: command.backup_dir,
                    },
                ),
            ),
            Command::Tree(command) => Some(Self::tree(
                command.workbook,
                command.depth,
                command.include_assets,
                sheet_selection,
                quiet,
            )),
            Command::Add(command) => Some(
                Self::mutation(
                    "add",
                    command.workbook,
                    command.mode.apply_mode.dry_run,
                    command.mode.backup,
                    sheet_selection,
                    quiet,
                )
                .with_action(Action::Add {
                    parent: command.parent,
                    title: command.title,
                    position: command.position,
                    create_missing_path: command.create_missing_path,
                }),
            ),
            Command::AddTree(command) => Some(
                Self::mutation(
                    "add-tree",
                    command.workbook,
                    command.mode.apply_mode.dry_run,
                    command.mode.backup,
                    sheet_selection,
                    quiet,
                )
                .with_action(Action::AddTree {
                    parent: command.parent,
                    input: command.input,
                    from_markdown: command.from_markdown,
                    markdown_mode: command.markdown_mode,
                }),
            ),
            Command::Set(command) => Some(
                Self::mutation(
                    "set",
                    command.workbook,
                    command.mode.apply_mode.dry_run,
                    command.mode.backup,
                    sheet_selection,
                    quiet,
                )
                .with_action(if let Some(title) = command.title {
                    Action::SetTitle {
                        node: command.node,
                        title,
                    }
                } else if let Some(note) = command.note {
                    Action::SetNote {
                        node: command.node,
                        note,
                    }
                } else if let Some(append_note) = command.append_note {
                    Action::SetAppendNote {
                        node: command.node,
                        append_note,
                    }
                } else if let Some(labels) = command.set_labels {
                    Action::SetLabels {
                        node: command.node,
                        labels: parse_csv_values(&labels),
                    }
                } else if let Some(label) = command.add_label {
                    Action::SetAddLabel {
                        node: command.node,
                        label,
                    }
                } else if let Some(label) = command.remove_label {
                    Action::SetRemoveLabel {
                        node: command.node,
                        label,
                    }
                } else if let Some(markers) = command.set_markers {
                    Action::SetMarkers {
                        node: command.node,
                        markers: parse_csv_values(&markers),
                    }
                } else if let Some(marker) = command.add_marker {
                    Action::SetAddMarker {
                        node: command.node,
                        marker,
                    }
                } else if let Some(marker) = command.remove_marker {
                    Action::SetRemoveMarker {
                        node: command.node,
                        marker,
                    }
                } else if let Some(hyperlink) = command.hyperlink {
                    Action::SetHyperlink {
                        node: command.node,
                        hyperlink,
                    }
                } else if !command.clear.is_empty() {
                    Action::SetClear {
                        node: command.node,
                        fields: command.clear,
                    }
                } else {
                    Action::Noop
                }),
            ),
            Command::Delete(command) => Some(
                Self::mutation(
                    "delete",
                    command.workbook,
                    command.mode.apply_mode.dry_run,
                    command.mode.backup,
                    sheet_selection,
                    quiet,
                )
                .with_action(Action::Delete {
                    node: command.node,
                    children_only: command.children_only,
                    promote_children: command.promote_children,
                }),
            ),
            Command::Move(command) => Some(
                Self::mutation(
                    "move",
                    command.workbook,
                    command.mode.apply_mode.dry_run,
                    command.mode.backup,
                    sheet_selection,
                    quiet,
                )
                .with_action(Action::Move {
                    node: command.node,
                    destination: command.to,
                    position: command.position,
                }),
            ),
            Command::Copy(command) => Some(
                Self::mutation(
                    "copy",
                    command.workbook,
                    command.mode.apply_mode.dry_run,
                    command.mode.backup,
                    sheet_selection,
                    quiet,
                )
                .with_action(Action::Copy {
                    node: command.node,
                    destination: command.to,
                    title: command.title,
                    position: command.position,
                    preserve_ids: command.preserve_ids,
                }),
            ),
            Command::Patch(command) => Some(Self::patch(
                command.workbook,
                command.mode.apply_mode.dry_run,
                command.mode.backup,
                command.ops,
                sheet_selection,
                quiet,
            )),
            Command::Import(command) => Some(Self::mutation(
                "import",
                command.workbook,
                command.mode.apply_mode.dry_run,
                command.mode.backup,
                sheet_selection,
                quiet,
            )),
            Command::Restore(command) => Some(Self::mutation(
                "restore",
                command.workbook,
                command.mode.apply_mode.dry_run,
                command.mode.backup,
                sheet_selection,
                quiet,
            )),
        }
    }

    fn read(command: &str, workbook: std::path::PathBuf, sheet_selection: SheetSelection) -> Self {
        Self {
            command: command.to_owned(),
            workbook,
            dry_run: false,
            applied: false,
            backup: false,
            quiet: false,
            no_color: false,
            sheet_selection,
            action: Action::Noop,
        }
    }

    fn tree(
        workbook: std::path::PathBuf,
        depth: Option<usize>,
        include_assets: bool,
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Self {
        let _ = include_assets;
        Self {
            command: "tree".to_owned(),
            workbook,
            dry_run: false,
            applied: false,
            backup: false,
            quiet,
            no_color: false,
            sheet_selection,
            action: Action::Tree { depth },
        }
    }

    fn inspect(workbook: std::path::PathBuf, sheet_selection: SheetSelection, quiet: bool) -> Self {
        Self {
            command: "inspect".to_owned(),
            workbook,
            dry_run: false,
            applied: false,
            backup: false,
            quiet,
            no_color: false,
            sheet_selection,
            action: Action::Inspect,
        }
    }

    fn sheets(workbook: std::path::PathBuf, sheet_selection: SheetSelection, quiet: bool) -> Self {
        Self {
            command: "sheets".to_owned(),
            workbook,
            dry_run: false,
            applied: false,
            backup: false,
            quiet,
            no_color: false,
            sheet_selection,
            action: Action::Sheets,
        }
    }

    fn get(
        workbook: std::path::PathBuf,
        node: String,
        depth: Option<usize>,
        include_assets: bool,
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Self {
        let _ = include_assets;
        Self {
            command: "get".to_owned(),
            workbook,
            dry_run: false,
            applied: false,
            backup: false,
            quiet,
            no_color: false,
            sheet_selection,
            action: Action::Get { node, depth },
        }
    }

    fn validate(
        workbook: std::path::PathBuf,
        strict: bool,
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Self {
        Self {
            command: "validate".to_owned(),
            workbook,
            dry_run: false,
            applied: false,
            backup: false,
            quiet,
            no_color: false,
            sheet_selection,
            action: Action::Validate { strict },
        }
    }

    fn find(
        workbook: std::path::PathBuf,
        args: FindInvocationArgs,
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Self {
        Self {
            command: "find".to_owned(),
            workbook,
            dry_run: false,
            applied: false,
            backup: false,
            quiet,
            no_color: false,
            sheet_selection,
            action: Action::Find {
                title: args.title,
                title_contains: args.title_contains,
                contains: args.contains,
                query: args.query,
                limit: args.limit,
                offset: args.offset,
            },
        }
    }

    fn mutation(
        command: &str,
        workbook: std::path::PathBuf,
        dry_run: bool,
        backup: bool,
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Self {
        Self {
            command: command.to_owned(),
            workbook,
            dry_run,
            applied: false,
            backup,
            quiet,
            no_color: false,
            sheet_selection,
            action: Action::Noop,
        }
    }

    fn with_action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    fn with_no_color(mut self, no_color: bool) -> Self {
        self.no_color = no_color;
        self
    }

    fn patch(
        workbook: std::path::PathBuf,
        dry_run: bool,
        backup: bool,
        ops: std::path::PathBuf,
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Self {
        Self {
            command: "patch".to_owned(),
            workbook,
            dry_run,
            applied: false,
            backup,
            quiet,
            no_color: false,
            sheet_selection,
            action: Action::Patch { ops },
        }
    }
}

fn render_inspect(
    invocation: Invocation,
    json: bool,
    fields: &[String],
    compact_json: bool,
) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };
    let workbook_path = invocation.workbook.display().to_string();

    let sheets = workbook
        .sheets
        .iter()
        .enumerate()
        .map(|(index, sheet)| SheetDto {
            id: sheet.id.0.clone(),
            index,
            title: sheet.title.clone(),
            root_topic_id: sheet.root.id.0.clone(),
            topic_count: count_topics(&sheet.root),
        })
        .collect::<Vec<_>>();

    let result = InspectResultDto {
        file: workbook_path.clone(),
        format: "xmind-zen",
        sheet_count: sheets.len(),
        sheets,
        resources_count: workbook.resources.len(),
        capabilities: InspectCapabilitiesDto {
            can_read_topics: true,
            can_preserve_unknown: !workbook.preservation.is_empty(),
        },
    };

    if json {
        let result = render_inspect_result(&result, fields, compact_json);
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(workbook_path),
            dry_run: false,
            applied: false,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!(
            "{}: {} sheets",
            invocation.workbook.display(),
            result.sheet_count
        );
    }

    0
}

fn render_sheets(invocation: Invocation, json: bool, fields: &[String], compact_json: bool) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let result = SheetsResultDto {
        sheets: workbook
            .sheets
            .iter()
            .enumerate()
            .map(|(index, sheet)| SheetDto {
                id: sheet.id.0.clone(),
                index,
                title: sheet.title.clone(),
                root_topic_id: sheet.root.id.0.clone(),
                topic_count: count_topics(&sheet.root),
            })
            .collect(),
    };

    if json {
        let result = render_sheets_result(&result, fields, compact_json);
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: false,
            applied: false,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        for sheet in result.sheets {
            println!("{}: {}", sheet.index, sheet.title);
        }
    }

    0
}

fn render_tree(
    invocation: Invocation,
    json: bool,
    depth: Option<usize>,
    fields: &[String],
    compact_json: bool,
) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let result = TreeResultDto::from_sheet(sheet, depth);

    if json {
        let result = render_tree_result(&result, fields, compact_json);
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: false,
            applied: false,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        render_tree_text(&result.root, 0);
    }

    0
}

fn render_get(
    invocation: Invocation,
    json: bool,
    node: &str,
    depth: Option<usize>,
    fields: &[String],
    compact_json: bool,
) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let result = GetResultDto {
        topic: TreeTopicDto::from_topic(resolved.topic, &resolved.path, depth),
    };

    if json {
        let result = render_get_result(&result, fields, compact_json);
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: false,
            applied: false,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        render_tree_text(&result.topic, 0);
    }

    0
}

fn render_find(invocation: Invocation, json: bool, options: FindRenderOptions) -> i32 {
    let criterion = match FindCriterion::from_args(
        options.title.as_deref(),
        options.title_contains.as_deref(),
        options.contains.as_deref(),
        options.query.as_deref(),
    ) {
        Ok(criterion) => criterion,
        Err(error) => return render_error(invocation, json, error),
    };

    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let mut matches = Vec::new();
    collect_find_matches(
        &sheet.root,
        &TopicPath::root(),
        &sheet.title,
        0,
        &criterion,
        &mut matches,
    );

    if let Some(offset) = options.offset {
        matches = matches.into_iter().skip(offset).collect();
    }

    if let Some(limit) = options.limit {
        matches.truncate(limit);
    }

    if json {
        let result = render_find_result(&matches, &options.fields, options.compact_json);
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: false,
            applied: false,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        for topic in matches {
            println!("{}\t{}", topic.path, topic.title);
        }
    }

    0
}

fn render_validate(invocation: Invocation, json: bool, strict: bool) -> i32 {
    if let Err(exit_code) = read_workbook_or_render_error(&invocation, json) {
        return exit_code;
    }
    let _strict = strict;

    let result = ValidateResultDto {
        valid: true,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: false,
            applied: false,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}: valid", invocation.workbook.display());
    }

    0
}

fn render_backup(
    invocation: Invocation,
    json: bool,
    backup_dir: Option<std::path::PathBuf>,
) -> i32 {
    let backup_dir = backup_dir.unwrap_or_else(|| {
        invocation
            .workbook
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".xmind-backups")
    });
    let timestamp = backup_timestamp();

    let backup = match create_backup_in_dir(&invocation.workbook, &backup_dir, timestamp) {
        Ok(backup) => backup,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::WriteFailed,
                format!("Backup could not be written: {error}"),
                true,
                "Check backup directory permissions and retry.",
            )
            .with_path(backup_dir.display().to_string());
            return render_error(invocation, json, error);
        }
    };

    let result = BackupResultDto {
        backup_path: backup.path.display().to_string(),
    };

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: false,
            applied: false,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}", result.backup_path);
    }

    0
}

fn backup_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

fn render_add(
    invocation: Invocation,
    json: bool,
    parent: &str,
    title: &str,
    position: Option<String>,
    create_missing_path: bool,
) -> i32 {
    let position = match parse_insert_position(position) {
        Ok(position) => position,
        Err(error) => return render_error(invocation, json, error),
    };

    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let parent_selector = match Selector::parse(parent) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Parent selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let parent = match resolve_topic(&sheet.root, &parent_selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            if create_missing_path {
                if let Selector::Path(path) = &parent_selector {
                    return render_add_create_missing_path(
                        invocation, json, sheet, path, title, position,
                    );
                }
            }
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "Parent selector did not match a topic: {}",
                    parent_selector.render()
                ),
                true,
                "Run tree or find to rediscover the parent selector, then retry.",
            )
            .with_selector(parent_selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Parent selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(parent_selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let position = match insert_position_from_spec(position, &parent, &sheet.root) {
        Ok(position) => position,
        Err(error) => return render_error(invocation, json, error),
    };

    let new_topic_id = generated_topic_id(title);
    let plan = MutationPlanner::plan_add_topic(AddTopicRequest {
        parent: parent.topic,
        parent_path: &parent.path,
        title,
        new_topic_id: &new_topic_id,
    });
    let created_path = plan.created_path.to_selector_value();
    let human_diff = plan.diff.clone();
    let mut result = AddDryRunResultDto {
        will_change: true,
        parent: TopicRefDto {
            id: parent.topic.id.0.clone(),
            path: parent.path.to_selector_value(),
            title: parent.topic.title.clone(),
        },
        created: CreatedTopicDto {
            path: created_path.clone(),
            title: plan.title.clone(),
        },
        summary: SummaryDto {
            added: 1,
            updated: 0,
            deleted: 0,
            moved: 0,
        },
        diff: vec![DiffEventDto {
            event: "added",
            path: created_path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::append_child_topic(
            &invocation.workbook,
            &plan.parent_id,
            &plan.title,
            &plan.new_topic_id,
            position,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}", render_human_outline(&human_diff));
    }

    0
}

fn render_add_create_missing_path(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    parent_path: &TopicPath,
    title: &str,
    position: PositionSpec,
) -> i32 {
    let Some((existing_parent, existing_parent_path, missing_segments)) =
        missing_path_segments(&sheet.root, parent_path)
    else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "create-missing-path requires a non-root path parent.",
            true,
            "Use --parent path:/Some/Missing/Path with --create-missing-path.",
        );
        return render_error(invocation, json, error);
    };
    let existing_resolved = ResolvedTopic {
        topic: existing_parent,
        path: existing_parent_path.clone(),
    };
    let position = match insert_position_from_spec(position, &existing_resolved, &sheet.root) {
        Ok(position) => position,
        Err(error) => return render_error(invocation, json, error),
    };

    let mut chain = missing_segments
        .iter()
        .map(|segment| (segment.clone(), generated_topic_id(segment)))
        .collect::<Vec<_>>();
    chain.push((title.to_owned(), generated_topic_id(title)));

    let mut created_paths = Vec::new();
    let mut current_path = existing_parent_path.clone();
    for (segment, _) in &chain {
        current_path = current_path.join(segment.clone());
        created_paths.push(current_path.to_selector_value());
    }

    let human_diff = Diff::from_events(
        created_paths
            .iter()
            .map(|path| DiffEvent::Added {
                path: TopicPath::parse_selector_value(path).expect("created path is valid"),
            })
            .collect(),
    );
    let mut result = AddDryRunResultDto {
        will_change: true,
        parent: TopicRefDto {
            id: existing_parent.id.0.clone(),
            path: existing_parent_path.to_selector_value(),
            title: existing_parent.title.clone(),
        },
        created: CreatedTopicDto {
            path: created_paths
                .last()
                .expect("created path includes final topic")
                .clone(),
            title: title.to_owned(),
        },
        summary: SummaryDto {
            added: created_paths.len(),
            updated: 0,
            deleted: 0,
            moved: 0,
        },
        diff: created_paths
            .into_iter()
            .map(|path| DiffEventDto {
                event: "added",
                path,
            })
            .collect(),
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::append_topic_chain(
            &invocation.workbook,
            &existing_parent.id.0,
            &chain,
            position,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}", render_human_outline(&human_diff));
    }

    0
}

fn render_add_tree(
    invocation: Invocation,
    json: bool,
    parent: &str,
    input: Option<&Path>,
    from_markdown: Option<&Path>,
    markdown_mode: Option<MarkdownMode>,
) -> i32 {
    if !invocation.dry_run {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "Only add-tree --dry-run is implemented for YAML tree input.",
            true,
            "Retry with --dry-run until the add-tree writer slice is implemented.",
        );
        return render_error(invocation, json, error);
    }

    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let parent_selector = match Selector::parse(parent) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Parent selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let parent = match resolve_topic(&sheet.root, &parent_selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "Parent selector did not match a topic: {}",
                    parent_selector.render()
                ),
                true,
                "Run tree or find to rediscover the parent selector, then retry.",
            )
            .with_selector(parent_selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Parent selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(parent_selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let tree = match read_tree_input(input, from_markdown, markdown_mode) {
        Ok(tree) => tree,
        Err(message) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidTreeInput,
                message,
                true,
                "Provide a YAML or JSON tree input file with a top-level title.",
            )
            .with_path(
                input
                    .or(from_markdown)
                    .map(Path::display)
                    .map(|path| path.to_string())
                    .unwrap_or_else(|| "<missing tree input>".to_owned()),
            );
            return render_error(invocation, json, error);
        }
    };
    if let Err(error) = validate_topic_tree_input(&tree) {
        let error = CliErrorBody::new(
            ErrorCode::InvalidTreeInput,
            error.message,
            true,
            "Provide a topic tree where every topic has a non-empty title.",
        )
        .with_path(
            input
                .or(from_markdown)
                .map(Path::display)
                .map(|path| path.to_string())
                .unwrap_or_else(|| "<missing tree input>".to_owned()),
        )
        .with_field_path(error.field_path);
        return render_error(invocation, json, error);
    }

    let added_paths = collect_added_paths(&parent.path, &tree);
    let created_root_path = added_paths
        .first()
        .expect("tree input creates at least the root topic")
        .clone();
    let result = AddTreeDryRunResultDto {
        will_change: !added_paths.is_empty(),
        parent: TopicRefDto {
            id: parent.topic.id.0.clone(),
            path: parent.path.to_selector_value(),
            title: parent.topic.title.clone(),
        },
        created_root: AddTreeCreatedTopicDto {
            id: tree.id.clone(),
            path: created_root_path,
            title: tree.title.clone(),
            note: tree.note.clone(),
            labels: tree.labels.clone(),
            markers: tree.markers.clone(),
            image: tree.image.clone(),
        },
        summary: SummaryDto {
            added: added_paths.len(),
            updated: 0,
            deleted: 0,
            moved: 0,
        },
        diff: added_paths
            .into_iter()
            .map(|path| DiffEventDto {
                event: "added",
                path,
            })
            .collect(),
    };

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: true,
            applied: false,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("planned {} added topics", result.summary.added);
    }

    0
}

fn generated_topic_id(title: &str) -> String {
    let slug = title
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();

    if slug.is_empty() {
        "topic-new".to_owned()
    } else {
        format!("topic-{slug}")
    }
}

fn parse_csv_values(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_clear_fields(fields: Vec<String>) -> Result<Vec<TopicClearField>, CliErrorBody> {
    let mut parsed = Vec::new();

    for field in fields {
        if field.contains(',') {
            return Err(CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Invalid --clear field: {field}"),
                true,
                "Pass one field per --clear flag; comma-separated clear fields are not supported.",
            ));
        }

        let clear_field = match field.as_str() {
            "note" => TopicClearField::Note,
            "labels" => TopicClearField::Labels,
            "markers" => TopicClearField::Markers,
            "hyperlink" => TopicClearField::Hyperlink,
            "image" => {
                return Err(CliErrorBody::new(
                    ErrorCode::InvalidUsage,
                    "Clearing topic images is not implemented yet.",
                    true,
                    "Use --clear note, --clear labels, --clear markers, or --clear hyperlink.",
                ));
            }
            _ => {
                return Err(CliErrorBody::new(
                    ErrorCode::InvalidUsage,
                    format!("Unknown --clear field: {field}"),
                    true,
                    "Use --clear note, --clear labels, --clear markers, or --clear hyperlink.",
                ));
            }
        };

        if !parsed.contains(&clear_field) {
            parsed.push(clear_field);
        }
    }

    Ok(parsed)
}

fn clear_field_name(field: TopicClearField) -> &'static str {
    match field {
        TopicClearField::Note => "note",
        TopicClearField::Labels => "labels",
        TopicClearField::Markers => "markers",
        TopicClearField::Hyperlink => "hyperlink",
    }
}

#[derive(Clone)]
enum PositionSpec {
    First,
    Last,
    Index(usize),
    Before(Selector),
    After(Selector),
}

fn parse_insert_position(position: Option<String>) -> Result<PositionSpec, CliErrorBody> {
    match position.as_deref() {
        None => Ok(PositionSpec::Last),
        Some("first") => Ok(PositionSpec::First),
        Some("last") => Ok(PositionSpec::Last),
        Some(index) if index.starts_with("index:") => {
            let value = index.trim_start_matches("index:");
            let index = value.parse::<usize>().map_err(|_| {
                CliErrorBody::new(
                    ErrorCode::InvalidUsage,
                    format!("Invalid position index: {value}"),
                    true,
                    "Use --position index:N where N is a non-negative integer.",
                )
            })?;
            Ok(PositionSpec::Index(index))
        }
        Some(before) if before.starts_with("before:") => {
            let value = before.trim_start_matches("before:");
            let selector = Selector::parse(value).map_err(|error| {
                CliErrorBody::new(
                    ErrorCode::InvalidUsage,
                    format!("Position selector is invalid: {error}"),
                    true,
                    "Use --position before:<selector> with a valid topic selector.",
                )
            })?;
            Ok(PositionSpec::Before(selector))
        }
        Some(after) if after.starts_with("after:") => {
            let value = after.trim_start_matches("after:");
            let selector = Selector::parse(value).map_err(|error| {
                CliErrorBody::new(
                    ErrorCode::InvalidUsage,
                    format!("Position selector is invalid: {error}"),
                    true,
                    "Use --position after:<selector> with a valid topic selector.",
                )
            })?;
            Ok(PositionSpec::After(selector))
        }
        Some(other) => Err(CliErrorBody::new(
            ErrorCode::InvalidUsage,
            format!("Unsupported position: {other}"),
            true,
            "Use --position first, --position last, --position index:N, --position before:<selector>, or --position after:<selector>.",
        )),
    }
}

fn insert_position_from_spec(
    position: PositionSpec,
    destination: &ResolvedTopic<'_>,
    root: &Topic,
) -> Result<InsertPosition, CliErrorBody> {
    match position {
        PositionSpec::First => Ok(InsertPosition::First),
        PositionSpec::Last => Ok(InsertPosition::Last),
        PositionSpec::Index(index) => {
            let child_count = destination.topic.children.len();
            if index > child_count {
                return Err(CliErrorBody::new(
                    ErrorCode::InvalidUsage,
                    format!("Position index {index} is outside the destination child range."),
                    true,
                    format!("Use an index between 0 and {child_count}."),
                ));
            }
            Ok(InsertPosition::Index(index))
        }
        PositionSpec::Before(selector) => {
            relative_insert_position(selector, false, destination, root)
        }
        PositionSpec::After(selector) => {
            relative_insert_position(selector, true, destination, root)
        }
    }
}

fn relative_insert_position(
    selector: Selector,
    after: bool,
    destination: &ResolvedTopic<'_>,
    root: &Topic,
) -> Result<InsertPosition, CliErrorBody> {
    match resolve_topic(root, &selector) {
        ResolveOne::Found(target) => {
            let Some(index) = destination
                .topic
                .children
                .iter()
                .position(|child| child.id.0 == target.topic.id.0)
            else {
                return Err(CliErrorBody::new(
                    ErrorCode::InvalidUsage,
                    "Position selector is not a child of the destination topic.",
                    true,
                    "Use a before:<selector> target that is already under the destination.",
                )
                .with_selector(selector.render()));
            };
            let index = if after { index + 1 } else { index };
            Ok(InsertPosition::Index(index))
        }
        ResolveOne::NotFound => Err(CliErrorBody::new(
            ErrorCode::NotFound,
            format!(
                "Position selector did not match a topic: {}",
                selector.render()
            ),
            true,
            "Run tree or find to rediscover the position selector, then retry.",
        )
        .with_selector(selector.render())),
        ResolveOne::Ambiguous(_) => Err(CliErrorBody::new(
            ErrorCode::AmbiguousSelector,
            "Position selector matched multiple topics.",
            true,
            "Use an id selector for --position before:<selector>.",
        )
        .with_selector(selector.render())),
    }
}

fn render_delete(
    invocation: Invocation,
    json: bool,
    node: &str,
    children_only: bool,
    promote_children: bool,
) -> i32 {
    if children_only && promote_children {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "delete cannot use --children-only and --promote-children together.",
            true,
            "Choose one delete mode and retry.",
        );
        return render_error(invocation, json, error);
    }

    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    if resolved.path.is_root() {
        let error = CliErrorBody::new(
            ErrorCode::RootOperationNotAllowed,
            "Deleting the root topic is not allowed.",
            true,
            "Use a non-root node selector.",
        )
        .with_selector(selector.render());
        return render_error(invocation, json, error);
    }

    let deleted = if children_only {
        collect_descendant_paths(resolved.topic, &resolved.path)
    } else if promote_children {
        vec![resolved.path.to_selector_value()]
    } else {
        collect_deleted_paths(resolved.topic, &resolved.path)
    };
    let promoted = if promote_children {
        collect_promoted_paths(resolved.topic, &resolved.path)
    } else {
        Vec::new()
    };
    let diff = deleted
        .iter()
        .cloned()
        .map(|path| DiffEventDto {
            event: "deleted",
            path,
        })
        .collect::<Vec<_>>();
    let mut result = DeleteDryRunResultDto {
        will_change: !deleted.is_empty(),
        deleted,
        summary: SummaryDto {
            added: 0,
            updated: 0,
            deleted: diff.len(),
            moved: promoted.len(),
        },
        diff,
        promoted,
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        let write_result = if children_only {
            crate::infra::xmind::encode::delete_topic_children(
                &invocation.workbook,
                &resolved.topic.id.0,
            )
        } else if promote_children {
            crate::infra::xmind::encode::delete_topic_promote_children(
                &invocation.workbook,
                &resolved.topic.id.0,
            )
        } else {
            crate::infra::xmind::encode::delete_topic(&invocation.workbook, &resolved.topic.id.0)
        };
        if let Err(error) = write_result {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        for path in &result.deleted {
            println!("- {path}");
        }
    }

    0
}

fn render_move(
    invocation: Invocation,
    json: bool,
    node: &str,
    destination: &str,
    position: Option<String>,
) -> i32 {
    let position = match parse_insert_position(position) {
        Ok(position) => position,
        Err(error) => return render_error(invocation, json, error),
    };

    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };
    let destination_selector = match Selector::parse(destination) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Destination selector is invalid: {error}"),
                true,
                "Use a valid destination selector such as root, id:<topic-id>, or path:/Q2.",
            );
            return render_error(invocation, json, error);
        }
    };

    let source = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    if source.path.is_root() {
        let error = CliErrorBody::new(
            ErrorCode::RootOperationNotAllowed,
            "Moving the root topic is not allowed.",
            true,
            "Use a non-root node selector.",
        )
        .with_selector(selector.render());
        return render_error(invocation, json, error);
    }

    let destination = match resolve_topic(&sheet.root, &destination_selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "Destination selector did not match a topic: {}",
                    destination_selector.render()
                ),
                true,
                "Run tree or find to rediscover the destination selector, then retry.",
            )
            .with_selector(destination_selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Destination selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(destination_selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let position = match insert_position_from_spec(position, &destination, &sheet.root) {
        Ok(position) => position,
        Err(error) => return render_error(invocation, json, error),
    };

    if destination
        .path
        .segments()
        .starts_with(source.path.segments())
    {
        let error = CliErrorBody::new(
            ErrorCode::PatchConflict,
            "Cannot move a topic into itself or one of its descendants.",
            true,
            "Choose a destination outside the source subtree.",
        )
        .with_selector(destination_selector.render());
        return render_error(invocation, json, error);
    }

    let from_path = source.path.to_selector_value();
    let to_path = destination
        .path
        .join(source.topic.title.clone())
        .to_selector_value();
    let mut result = MoveDryRunResultDto {
        will_change: from_path != to_path,
        moved: MovedTopicDto {
            id: source.topic.id.0.clone(),
            from_path: from_path.clone(),
            to_path: to_path.clone(),
        },
        summary: SummaryDto {
            added: 0,
            updated: 0,
            deleted: 0,
            moved: usize::from(from_path != to_path),
        },
        diff: vec![MoveDiffEventDto {
            event: "moved",
            from: from_path,
            to: to_path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::move_topic(
            &invocation.workbook,
            &source.topic.id.0,
            &destination.topic.id.0,
            position,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        for event in &result.diff {
            println!("> {} -> {}", event.from, event.to);
        }
    }

    0
}

fn render_copy(
    invocation: Invocation,
    json: bool,
    node: &str,
    destination: &str,
    title: Option<String>,
    position: Option<String>,
    preserve_ids: bool,
) -> i32 {
    if preserve_ids {
        let error = CliErrorBody::new(
            ErrorCode::PatchConflict,
            "copy --preserve-ids would create duplicate topic ids in the same workbook.",
            true,
            "Omit --preserve-ids when copying within the same workbook.",
        );
        return render_error(invocation, json, error);
    }

    let position = match parse_insert_position(position) {
        Ok(position) => position,
        Err(error) => return render_error(invocation, json, error),
    };

    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };
    let destination_selector = match Selector::parse(destination) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Destination selector is invalid: {error}"),
                true,
                "Use a valid destination selector such as root, id:<topic-id>, or path:/Q2.",
            );
            return render_error(invocation, json, error);
        }
    };

    let source = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    if source.path.is_root() {
        let error = CliErrorBody::new(
            ErrorCode::RootOperationNotAllowed,
            "Copying the root topic is not allowed.",
            true,
            "Use a non-root node selector.",
        )
        .with_selector(selector.render());
        return render_error(invocation, json, error);
    }

    let destination = match resolve_topic(&sheet.root, &destination_selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "Destination selector did not match a topic: {}",
                    destination_selector.render()
                ),
                true,
                "Run tree or find to rediscover the destination selector, then retry.",
            )
            .with_selector(destination_selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Destination selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(destination_selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let position = match insert_position_from_spec(position, &destination, &sheet.root) {
        Ok(position) => position,
        Err(error) => return render_error(invocation, json, error),
    };

    let copied_title = title.unwrap_or_else(|| source.topic.title.clone());
    if copied_title.is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "copy title cannot be empty.",
            true,
            "Omit --title or provide a non-empty title.",
        );
        return render_error(invocation, json, error);
    }

    let new_id = format!("{}-copy", source.topic.id.0);
    let copied_paths = collect_copied_paths(source.topic, &destination.path, &copied_title);
    let copied_path = copied_paths
        .first()
        .expect("copied paths include copied root")
        .clone();
    let added = count_topics(source.topic);
    let mut result = CopyDryRunResultDto {
        will_change: true,
        copied_root: CopiedRootDto {
            source_id: source.topic.id.0.clone(),
            new_id: new_id.clone(),
            path: copied_path.clone(),
        },
        summary: SummaryDto {
            added,
            updated: 0,
            deleted: 0,
            moved: 0,
        },
        diff: copied_paths
            .into_iter()
            .map(|path| DiffEventDto {
                event: "added",
                path,
            })
            .collect(),
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::copy_topic(
            &invocation.workbook,
            &source.topic.id.0,
            &destination.topic.id.0,
            &new_id,
            &copied_title,
            position,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        for event in &result.diff {
            println!("+ {}", event.path);
        }
    }

    0
}

fn render_set_title(invocation: Invocation, json: bool, node: &str, title: &str) -> i32 {
    if title.is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "set requires at least one field option.",
            true,
            "Retry with --title for this implementation slice.",
        );
        return render_error(invocation, json, error);
    }

    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let old_path = resolved.path.to_selector_value();
    let new_path = renamed_path(&resolved.path, title);
    let human_diff = Diff::from_events(vec![DiffEvent::Updated {
        path: TopicPath::parse_selector_value(&new_path)
            .expect("renamed topic path remains a valid absolute path"),
        fields: vec![FieldChange::new("title")],
    }]);
    let mut result = SetTitleDryRunResultDto {
        will_change: resolved.topic.title != title,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: None,
            old_path: Some(old_path),
            new_path: Some(new_path.clone()),
            new_note: None,
            new_labels: None,
            new_markers: None,
            new_hyperlink: None,
            changed_fields: vec!["title"],
        },
        summary: SummaryDto {
            added: 0,
            updated: 1,
            deleted: 0,
            moved: 0,
        },
        diff: vec![DiffEventDto {
            event: "updated",
            path: new_path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::rename_topic(
            &invocation.workbook,
            &resolved.topic.id.0,
            title,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}", render_human_outline(&human_diff));
    }

    0
}

pub(super) fn renamed_path(path: &TopicPath, title: &str) -> String {
    if path.is_root() {
        return TopicPath::root().join(title.to_owned()).to_selector_value();
    }

    let mut segments = path.segments().to_vec();
    if let Some(last) = segments.last_mut() {
        *last = title.to_owned();
    }
    TopicPath::from_segments(segments).to_selector_value()
}

fn render_set_note(invocation: Invocation, json: bool, node: &str, note: &str) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let path = resolved.path.to_selector_value();
    let mut result = SetTitleDryRunResultDto {
        will_change: true,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: Some(path.clone()),
            old_path: None,
            new_path: None,
            new_note: None,
            new_labels: None,
            new_markers: None,
            new_hyperlink: None,
            changed_fields: vec!["note"],
        },
        summary: SummaryDto {
            added: 0,
            updated: 1,
            deleted: 0,
            moved: 0,
        },
        diff: vec![DiffEventDto {
            event: "updated",
            path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::set_topic_note(
            &invocation.workbook,
            &resolved.topic.id.0,
            note,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("planned 1 updated topic");
    }

    0
}

fn render_set_labels(invocation: Invocation, json: bool, node: &str, labels: Vec<String>) -> i32 {
    render_set_label_mutation(invocation, json, node, LabelMutation::Replace(labels))
}

fn render_set_add_label(invocation: Invocation, json: bool, node: &str, label: String) -> i32 {
    render_set_label_mutation(invocation, json, node, LabelMutation::Add(label))
}

fn render_set_remove_label(invocation: Invocation, json: bool, node: &str, label: String) -> i32 {
    render_set_label_mutation(invocation, json, node, LabelMutation::Remove(label))
}

enum LabelMutation {
    Replace(Vec<String>),
    Add(String),
    Remove(String),
}

fn render_set_label_mutation(
    invocation: Invocation,
    json: bool,
    node: &str,
    mutation: LabelMutation,
) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let labels = match mutation {
        LabelMutation::Replace(labels) => labels,
        LabelMutation::Add(label) => {
            let mut labels = resolved.topic.labels.clone();
            if !labels.iter().any(|existing| existing == &label) {
                labels.push(label);
            }
            labels
        }
        LabelMutation::Remove(label) => resolved
            .topic
            .labels
            .iter()
            .filter(|existing| *existing != &label)
            .cloned()
            .collect(),
    };
    let path = resolved.path.to_selector_value();
    let human_diff = Diff::from_events(vec![DiffEvent::Updated {
        path: resolved.path.clone(),
        fields: vec![FieldChange::new("labels")],
    }]);
    let mut result = SetTitleDryRunResultDto {
        will_change: resolved.topic.labels != labels,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: Some(path.clone()),
            old_path: None,
            new_path: None,
            new_note: None,
            new_labels: Some(labels),
            new_markers: None,
            new_hyperlink: None,
            changed_fields: vec!["labels"],
        },
        summary: SummaryDto {
            added: 0,
            updated: 1,
            deleted: 0,
            moved: 0,
        },
        diff: vec![DiffEventDto {
            event: "updated",
            path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::set_topic_labels(
            &invocation.workbook,
            &resolved.topic.id.0,
            result
                .updated
                .new_labels
                .as_ref()
                .expect("set-labels result carries new labels"),
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}", render_human_outline(&human_diff));
    }

    0
}

fn render_set_markers(invocation: Invocation, json: bool, node: &str, markers: Vec<String>) -> i32 {
    render_set_marker_mutation(invocation, json, node, MarkerMutation::Replace(markers))
}

fn render_set_add_marker(invocation: Invocation, json: bool, node: &str, marker: String) -> i32 {
    render_set_marker_mutation(invocation, json, node, MarkerMutation::Add(marker))
}

fn render_set_remove_marker(invocation: Invocation, json: bool, node: &str, marker: String) -> i32 {
    render_set_marker_mutation(invocation, json, node, MarkerMutation::Remove(marker))
}

enum MarkerMutation {
    Replace(Vec<String>),
    Add(String),
    Remove(String),
}

fn render_set_marker_mutation(
    invocation: Invocation,
    json: bool,
    node: &str,
    mutation: MarkerMutation,
) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let markers = match mutation {
        MarkerMutation::Replace(markers) => markers,
        MarkerMutation::Add(marker) => {
            let mut markers = resolved.topic.markers.clone();
            if !markers.iter().any(|existing| existing == &marker) {
                markers.push(marker);
            }
            markers
        }
        MarkerMutation::Remove(marker) => resolved
            .topic
            .markers
            .iter()
            .filter(|existing| *existing != &marker)
            .cloned()
            .collect(),
    };
    let path = resolved.path.to_selector_value();
    let human_diff = Diff::from_events(vec![DiffEvent::Updated {
        path: resolved.path.clone(),
        fields: vec![FieldChange::new("markers")],
    }]);
    let mut result = SetTitleDryRunResultDto {
        will_change: resolved.topic.markers != markers,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: Some(path.clone()),
            old_path: None,
            new_path: None,
            new_note: None,
            new_labels: None,
            new_markers: Some(markers),
            new_hyperlink: None,
            changed_fields: vec!["markers"],
        },
        summary: SummaryDto {
            added: 0,
            updated: 1,
            deleted: 0,
            moved: 0,
        },
        diff: vec![DiffEventDto {
            event: "updated",
            path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::set_topic_markers(
            &invocation.workbook,
            &resolved.topic.id.0,
            result
                .updated
                .new_markers
                .as_ref()
                .expect("set-markers result carries new markers"),
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}", render_human_outline(&human_diff));
    }

    0
}

fn render_set_hyperlink(invocation: Invocation, json: bool, node: &str, hyperlink: &str) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let path = resolved.path.to_selector_value();
    let human_diff = Diff::from_events(vec![DiffEvent::Updated {
        path: resolved.path.clone(),
        fields: vec![FieldChange::new("hyperlink")],
    }]);
    let mut result = SetTitleDryRunResultDto {
        will_change: resolved.topic.hyperlink.as_deref() != Some(hyperlink),
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: Some(path.clone()),
            old_path: None,
            new_path: None,
            new_note: None,
            new_labels: None,
            new_markers: None,
            new_hyperlink: Some(hyperlink.to_owned()),
            changed_fields: vec!["hyperlink"],
        },
        summary: SummaryDto {
            added: 0,
            updated: 1,
            deleted: 0,
            moved: 0,
        },
        diff: vec![DiffEventDto {
            event: "updated",
            path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::set_topic_hyperlink(
            &invocation.workbook,
            &resolved.topic.id.0,
            hyperlink,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}", render_human_outline(&human_diff));
    }

    0
}

fn render_set_clear(invocation: Invocation, json: bool, node: &str, fields: Vec<String>) -> i32 {
    let clear_fields = match parse_clear_fields(fields) {
        Ok(fields) => fields,
        Err(error) => return render_error(invocation, json, error),
    };

    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let changed_fields = clear_fields
        .iter()
        .copied()
        .map(clear_field_name)
        .collect::<Vec<_>>();
    let will_change = clear_fields.iter().any(|field| match field {
        TopicClearField::Note => resolved.topic.note.is_some(),
        TopicClearField::Labels => !resolved.topic.labels.is_empty(),
        TopicClearField::Markers => !resolved.topic.markers.is_empty(),
        TopicClearField::Hyperlink => resolved.topic.hyperlink.is_some(),
    });
    let path = resolved.path.to_selector_value();
    let human_diff = Diff::from_events(vec![DiffEvent::Updated {
        path: resolved.path.clone(),
        fields: changed_fields
            .iter()
            .map(|field| FieldChange::new(*field))
            .collect(),
    }]);
    let mut result = SetTitleDryRunResultDto {
        will_change,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: Some(path.clone()),
            old_path: None,
            new_path: None,
            new_note: None,
            new_labels: None,
            new_markers: None,
            new_hyperlink: None,
            changed_fields,
        },
        summary: SummaryDto {
            added: 0,
            updated: 1,
            deleted: 0,
            moved: 0,
        },
        diff: vec![DiffEventDto {
            event: "updated",
            path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::clear_topic_fields(
            &invocation.workbook,
            &resolved.topic.id.0,
            &clear_fields,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("{}", render_human_outline(&human_diff));
    }

    0
}

fn render_set_append_note(
    invocation: Invocation,
    json: bool,
    node: &str,
    append_note: &str,
) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                format!("Node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            );
            return render_error(invocation, json, error);
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("Selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the topic selector, then retry.",
            )
            .with_selector(selector.render());
            return render_error(invocation, json, error);
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "Selector matched multiple topics.",
                true,
                "Retry with one of the candidate ids.",
            )
            .with_selector(selector.render())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|candidate| CandidateDto {
                        id: candidate.topic.id.0.clone(),
                        path: candidate.path.to_selector_value(),
                        title: candidate.topic.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            return render_error(invocation, json, error);
        }
    };

    let path = resolved.path.to_selector_value();
    let new_note = format!(
        "{}{}",
        resolved.topic.note.as_deref().unwrap_or(""),
        append_note
    );
    let mut result = SetTitleDryRunResultDto {
        will_change: true,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: Some(path.clone()),
            old_path: None,
            new_path: None,
            new_note: Some(new_note.clone()),
            new_labels: None,
            new_markers: None,
            new_hyperlink: None,
            changed_fields: vec!["note"],
        },
        summary: SummaryDto {
            added: 0,
            updated: 1,
            deleted: 0,
            moved: 0,
        },
        diff: vec![DiffEventDto {
            event: "updated",
            path,
        }],
        backup_path: None,
    };

    if !invocation.dry_run {
        result.backup_path = match create_mutation_backup(&invocation) {
            Ok(backup_path) => backup_path,
            Err(error) => return render_backup_error(invocation, json, error),
        };
        if let Err(error) = crate::infra::xmind::encode::set_topic_note(
            &invocation.workbook,
            &resolved.topic.id.0,
            &new_note,
        ) {
            return render_workbook_write_error(invocation, json, error);
        }
    }

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: !invocation.dry_run,
            result: Some(result),
            error: None,
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else if !invocation.quiet {
        println!("planned 1 updated topic");
    }

    0
}

#[derive(Debug, Serialize)]
struct ValidateResultDto {
    valid: bool,
    warnings: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BackupResultDto {
    backup_path: String,
}

pub(super) fn read_workbook_or_render_error(
    invocation: &Invocation,
    json: bool,
) -> Result<crate::domain::workbook::Workbook, i32> {
    crate::infra::xmind::decode::read_workbook(&invocation.workbook).map_err(|error| {
        let path = invocation.workbook.display().to_string();
        let error = match error {
            crate::infra::xmind::decode::XMindReadError::UnsupportedFormat => CliErrorBody::new(
                ErrorCode::UnsupportedFormat,
                "Workbook uses an unsupported XMind format variant.",
                false,
                "Open and re-save the file with a supported XMind version, or use export/import.",
            )
            .with_path(path),
            other => CliErrorBody::new(
                ErrorCode::ParseFailed,
                format!("Workbook could not be parsed: {other}"),
                false,
                "Open and re-save the workbook with a supported XMind version, then retry.",
            )
            .with_path(path),
        };
        render_error(invocation.clone(), json, error)
    })
}

pub(super) fn select_sheet_or_render_error<'a>(
    workbook: &'a crate::domain::workbook::Workbook,
    invocation: &Invocation,
    json: bool,
) -> Result<&'a Sheet, i32> {
    match select_sheet(workbook, &invocation.sheet_selection) {
        SheetResolution::Found(sheet) => Ok(sheet),
        SheetResolution::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::SheetNotFound,
                "Sheet selector did not match any sheet.",
                true,
                "Run sheets to list available sheet titles, ids, and indexes, then retry.",
            )
            .with_details(invocation.sheet_selection.to_details());
            Err(render_error(invocation.clone(), json, error))
        }
        SheetResolution::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSheet,
                "Sheet selector matched multiple sheets.",
                true,
                "Retry with one of the candidate sheet ids.",
            )
            .with_details(invocation.sheet_selection.to_details())
            .with_candidates(
                candidates
                    .into_iter()
                    .map(|sheet| CandidateDto {
                        id: sheet.id.0.clone(),
                        path: "/".to_owned(),
                        title: sheet.title.clone(),
                        sheet: Some(sheet.title.clone()),
                    })
                    .collect(),
            );
            Err(render_error(invocation.clone(), json, error))
        }
    }
}

enum SheetResolution<'a> {
    Found(&'a Sheet),
    NotFound,
    Ambiguous(Vec<&'a Sheet>),
}

fn select_sheet<'a>(
    workbook: &'a crate::domain::workbook::Workbook,
    selection: &SheetSelection,
) -> SheetResolution<'a> {
    if let Some(index) = selection.index {
        return workbook
            .sheets
            .get(index)
            .map_or(SheetResolution::NotFound, SheetResolution::Found);
    }

    if let Some(id) = &selection.id {
        return workbook
            .sheets
            .iter()
            .find(|sheet| sheet.id.0 == id.as_str())
            .map_or(SheetResolution::NotFound, SheetResolution::Found);
    }

    if let Some(title) = &selection.title {
        let matches = workbook
            .sheets
            .iter()
            .filter(|sheet| sheet.title == title.as_str())
            .collect::<Vec<_>>();

        return match matches.as_slice() {
            [] => SheetResolution::NotFound,
            [sheet] => SheetResolution::Found(sheet),
            _ => SheetResolution::Ambiguous(matches),
        };
    }

    workbook
        .sheets
        .first()
        .map_or(SheetResolution::NotFound, SheetResolution::Found)
}

impl SheetSelection {
    fn to_details(&self) -> Value {
        if let Some(index) = self.index {
            return serde_json::json!({ "sheet_index": index });
        }

        if let Some(id) = &self.id {
            return serde_json::json!({ "sheet_id": id });
        }

        if let Some(title) = &self.title {
            return serde_json::json!({ "sheet": title });
        }

        Value::Null
    }
}

#[derive(Debug, Serialize)]
struct SheetsResultDto {
    sheets: Vec<SheetDto>,
}

#[derive(Debug, Serialize)]
struct InspectResultDto {
    file: String,
    format: &'static str,
    sheet_count: usize,
    sheets: Vec<SheetDto>,
    resources_count: usize,
    capabilities: InspectCapabilitiesDto,
}

#[derive(Debug, Serialize)]
struct InspectCapabilitiesDto {
    can_read_topics: bool,
    can_preserve_unknown: bool,
}

#[derive(Debug, Serialize)]
struct SheetDto {
    id: String,
    index: usize,
    title: String,
    root_topic_id: String,
    topic_count: usize,
}

#[derive(Debug, Serialize)]
struct AddTreeDryRunResultDto {
    will_change: bool,
    parent: TopicRefDto,
    created_root: AddTreeCreatedTopicDto,
    summary: SummaryDto,
    diff: Vec<DiffEventDto>,
}

#[derive(Debug, Serialize)]
struct AddDryRunResultDto {
    will_change: bool,
    parent: TopicRefDto,
    created: CreatedTopicDto,
    summary: SummaryDto,
    diff: Vec<DiffEventDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backup_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeleteDryRunResultDto {
    will_change: bool,
    deleted: Vec<String>,
    summary: SummaryDto,
    diff: Vec<DiffEventDto>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    promoted: Vec<PromotedTopicDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backup_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct PromotedTopicDto {
    id: String,
    from_path: String,
    to_path: String,
}

#[derive(Debug, Serialize)]
struct MoveDryRunResultDto {
    will_change: bool,
    moved: MovedTopicDto,
    summary: SummaryDto,
    diff: Vec<MoveDiffEventDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backup_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct CopyDryRunResultDto {
    will_change: bool,
    copied_root: CopiedRootDto,
    summary: SummaryDto,
    diff: Vec<DiffEventDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backup_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct CopiedRootDto {
    source_id: String,
    new_id: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct MovedTopicDto {
    id: String,
    from_path: String,
    to_path: String,
}

#[derive(Debug, Serialize)]
struct MoveDiffEventDto {
    event: &'static str,
    from: String,
    to: String,
}

#[derive(Debug, Serialize)]
struct TopicRefDto {
    id: String,
    path: String,
    title: String,
}

#[derive(Debug, Serialize)]
struct CreatedTopicDto {
    path: String,
    title: String,
}

#[derive(Debug, Serialize)]
struct AddTreeCreatedTopicDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    path: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    labels: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    markers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<TopicTreeImageInputDto>,
}

#[derive(Debug, Serialize)]
struct SetTitleDryRunResultDto {
    will_change: bool,
    updated: UpdatedTopicDto,
    summary: SummaryDto,
    diff: Vec<DiffEventDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backup_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct UpdatedTopicDto {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    old_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_markers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_hyperlink: Option<String>,
    changed_fields: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct SummaryDto {
    added: usize,
    updated: usize,
    deleted: usize,
    moved: usize,
}

#[derive(Debug, Serialize)]
struct DiffEventDto {
    event: &'static str,
    path: String,
}

#[derive(Debug, Serialize)]
struct GetResultDto {
    topic: TreeTopicDto,
}

fn render_inspect_result(
    result: &InspectResultDto,
    fields: &[String],
    compact_json: bool,
) -> Value {
    if !compact_json {
        return serde_json::to_value(result).expect("inspect result serializes");
    }

    let requested_fields = if fields.is_empty() {
        vec!["file", "format", "sheet_count", "sheets", "capabilities"]
    } else {
        fields.iter().map(String::as_str).collect::<Vec<_>>()
    };

    let mut object = serde_json::Map::new();

    for field in requested_fields {
        match field {
            "file" => {
                object.insert("file".to_owned(), serde_json::json!(result.file));
            }
            "format" => {
                object.insert("format".to_owned(), serde_json::json!(result.format));
            }
            "sheet_count" => {
                object.insert(
                    "sheet_count".to_owned(),
                    serde_json::json!(result.sheet_count),
                );
            }
            "sheets" => {
                object.insert("sheets".to_owned(), serde_json::json!(result.sheets));
            }
            "resources_count" => {
                object.insert(
                    "resources_count".to_owned(),
                    serde_json::json!(result.resources_count),
                );
            }
            "capabilities" => {
                object.insert(
                    "capabilities".to_owned(),
                    serde_json::json!(result.capabilities),
                );
            }
            _ => {}
        }
    }

    Value::Object(object)
}

fn render_sheets_result(result: &SheetsResultDto, fields: &[String], compact_json: bool) -> Value {
    if !compact_json {
        return serde_json::to_value(result).expect("sheets result serializes");
    }

    let requested_fields = if fields.is_empty() {
        vec!["id", "index", "title", "topic_count"]
    } else {
        fields.iter().map(String::as_str).collect::<Vec<_>>()
    };

    serde_json::json!({
        "sheets": result
            .sheets
            .iter()
            .map(|sheet| render_sheet(sheet, &requested_fields))
            .collect::<Vec<_>>()
    })
}

fn render_sheet(sheet: &SheetDto, fields: &[&str]) -> Value {
    let mut object = serde_json::Map::new();

    for field in fields {
        match *field {
            "id" => {
                object.insert("id".to_owned(), serde_json::json!(sheet.id));
            }
            "index" => {
                object.insert("index".to_owned(), serde_json::json!(sheet.index));
            }
            "title" => {
                object.insert("title".to_owned(), serde_json::json!(sheet.title));
            }
            "root_topic_id" => {
                object.insert(
                    "root_topic_id".to_owned(),
                    serde_json::json!(sheet.root_topic_id),
                );
            }
            "topic_count" => {
                object.insert(
                    "topic_count".to_owned(),
                    serde_json::json!(sheet.topic_count),
                );
            }
            _ => {}
        }
    }

    Value::Object(object)
}

fn render_get_result(result: &GetResultDto, fields: &[String], compact_json: bool) -> Value {
    if !compact_json {
        return serde_json::to_value(result).expect("get result serializes");
    }

    let requested_fields = if fields.is_empty() {
        vec![
            "id",
            "path",
            "title",
            "note",
            "labels",
            "markers",
            "hyperlink",
            "image",
            "children_count",
        ]
    } else {
        fields.iter().map(String::as_str).collect::<Vec<_>>()
    };

    serde_json::json!({
        "topic": render_tree_topic(&result.topic, &requested_fields),
    })
}

#[derive(Debug, Serialize)]
struct FindMatchDto {
    id: String,
    path: String,
    title: String,
    sheet: String,
    children_count: usize,
}

struct FindRenderOptions {
    title: Option<String>,
    title_contains: Option<String>,
    contains: Option<String>,
    query: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    fields: Vec<String>,
    compact_json: bool,
}

enum FindCriterion<'a> {
    TitleExact(&'a str),
    TitleContains(&'a str),
    Contains(&'a str),
    Query(QueryExpr),
}

impl<'a> FindCriterion<'a> {
    fn from_args(
        title: Option<&'a str>,
        title_contains: Option<&'a str>,
        contains: Option<&'a str>,
        query: Option<&'a str>,
    ) -> Result<Self, CliErrorBody> {
        match (title, title_contains, contains, query) {
            (Some(title), None, None, None) => Ok(Self::TitleExact(title)),
            (None, Some(title_contains), None, None) => Ok(Self::TitleContains(title_contains)),
            (None, None, Some(contains), None) => Ok(Self::Contains(contains)),
            (None, None, None, Some(query)) => {
                let expr = QueryExpr::parse(query).map_err(|error| {
                    CliErrorBody::new(
                        ErrorCode::InvalidUsage,
                        format!("find query is invalid: {error}"),
                        true,
                        "Use a valid query expression such as title = \"Payment\".",
                    )
                })?;
                Ok(Self::Query(expr))
            }
            (None, None, None, None) => Err(CliErrorBody::new(
                ErrorCode::InvalidUsage,
                "find requires a search criterion.",
                true,
                "Retry with --title, --title-contains, --contains, or --query.",
            )),
            _ => Err(CliErrorBody::new(
                ErrorCode::InvalidUsage,
                "find accepts only one search criterion at a time.",
                true,
                "Retry with only one of --title, --title-contains, --contains, or --query.",
            )),
        }
    }

    fn matches_topic(&self, topic: &Topic, path: &TopicPath, depth: usize) -> bool {
        match self {
            Self::TitleExact(expected) => topic.title == *expected,
            Self::TitleContains(needle) => topic.title.contains(needle),
            Self::Contains(needle) => {
                topic.title.contains(needle)
                    || topic
                        .note
                        .as_deref()
                        .is_some_and(|note| note.contains(needle))
            }
            Self::Query(expr) => expr.matches_topic(topic, path, depth),
        }
    }
}

fn render_find_result(matches: &[FindMatchDto], fields: &[String], compact_json: bool) -> Value {
    let requested_fields = if compact_json && !fields.is_empty() {
        fields.iter().map(String::as_str).collect::<Vec<_>>()
    } else {
        vec!["id", "path", "title", "sheet", "children_count"]
    };

    serde_json::json!({
        "matches": matches
            .iter()
            .map(|topic| render_find_match(topic, &requested_fields))
            .collect::<Vec<_>>()
    })
}

fn render_find_match(topic: &FindMatchDto, fields: &[&str]) -> Value {
    let mut object = serde_json::Map::new();

    for field in fields {
        match *field {
            "id" => {
                object.insert("id".to_owned(), serde_json::json!(topic.id));
            }
            "path" => {
                object.insert("path".to_owned(), serde_json::json!(topic.path));
            }
            "title" => {
                object.insert("title".to_owned(), serde_json::json!(topic.title));
            }
            "sheet" => {
                object.insert("sheet".to_owned(), serde_json::json!(topic.sheet));
            }
            "children_count" => {
                object.insert(
                    "children_count".to_owned(),
                    serde_json::json!(topic.children_count),
                );
            }
            _ => {}
        }
    }

    Value::Object(object)
}

#[derive(Debug, Serialize)]
struct TreeResultDto {
    sheet: String,
    root: TreeTopicDto,
}

impl TreeResultDto {
    fn from_sheet(sheet: &Sheet, depth: Option<usize>) -> Self {
        Self {
            sheet: sheet.title.clone(),
            root: TreeTopicDto::from_topic(&sheet.root, &TopicPath::root(), depth),
        }
    }
}

fn render_tree_result(result: &TreeResultDto, fields: &[String], compact_json: bool) -> Value {
    if !compact_json {
        return serde_json::to_value(result).expect("tree result serializes");
    }

    let requested_fields = if fields.is_empty() {
        vec!["id", "path", "title", "children_count", "children"]
    } else {
        fields.iter().map(String::as_str).collect::<Vec<_>>()
    };

    serde_json::json!({
        "sheet": result.sheet,
        "root": render_tree_topic(&result.root, &requested_fields),
    })
}

fn render_tree_topic(topic: &TreeTopicDto, fields: &[&str]) -> Value {
    let mut object = serde_json::Map::new();

    for field in fields {
        match *field {
            "id" => {
                object.insert("id".to_owned(), serde_json::json!(topic.id));
            }
            "path" => {
                object.insert("path".to_owned(), serde_json::json!(topic.path));
            }
            "title" => {
                object.insert("title".to_owned(), serde_json::json!(topic.title));
            }
            "note" => {
                if let Some(note) = &topic.note {
                    object.insert("note".to_owned(), serde_json::json!(note));
                }
            }
            "labels" => {
                object.insert("labels".to_owned(), serde_json::json!(topic.labels));
            }
            "markers" => {
                object.insert("markers".to_owned(), serde_json::json!(topic.markers));
            }
            "hyperlink" => {
                if let Some(hyperlink) = &topic.hyperlink {
                    object.insert("hyperlink".to_owned(), serde_json::json!(hyperlink));
                }
            }
            "image" => {
                if let Some(image) = &topic.image {
                    object.insert("image".to_owned(), serde_json::json!(image));
                }
            }
            "children_count" => {
                if let Some(children_count) = topic.children_count {
                    object.insert(
                        "children_count".to_owned(),
                        serde_json::json!(children_count),
                    );
                }
            }
            "children" => {
                if let Some(children) = &topic.children {
                    object.insert(
                        "children".to_owned(),
                        Value::Array(
                            children
                                .iter()
                                .map(|child| render_tree_topic(child, fields))
                                .collect(),
                        ),
                    );
                }
            }
            _ => {}
        }
    }

    Value::Object(object)
}

#[derive(Debug, Serialize)]
struct TreeTopicDto {
    id: String,
    path: String,
    title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,

    labels: Vec<String>,

    markers: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    hyperlink: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<TopicImageRefDto>,

    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<TreeTopicDto>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    children_count: Option<usize>,
}

impl TreeTopicDto {
    fn from_topic(topic: &Topic, path: &TopicPath, remaining_depth: Option<usize>) -> Self {
        let include_children = remaining_depth.map_or(true, |depth| depth > 0);
        let next_depth = remaining_depth.map(|depth| depth.saturating_sub(1));
        let children = include_children.then(|| {
            topic
                .children
                .iter()
                .map(|child| {
                    let child_path = path.join(child.title.clone());
                    Self::from_topic(child, &child_path, next_depth)
                })
                .collect()
        });

        Self {
            id: topic.id.0.clone(),
            path: path.to_selector_value(),
            title: topic.title.clone(),
            note: topic.note.clone(),
            labels: topic.labels.clone(),
            markers: topic.markers.clone(),
            hyperlink: topic.hyperlink.clone(),
            image: topic.image.as_ref().map(TopicImageRefDto::from_topic_image),
            children,
            children_count: (!include_children && !topic.children.is_empty())
                .then_some(topic.children.len()),
        }
    }
}

#[derive(Debug, Serialize)]
struct TopicImageRefDto {
    asset_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    alt: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

impl TopicImageRefDto {
    fn from_topic_image(image: &crate::domain::topic::TopicImageRef) -> Self {
        Self {
            asset_id: image.asset_id.as_str().to_owned(),
            alt: image.alt.clone(),
            title: image.title.clone(),
        }
    }
}

fn render_tree_text(topic: &TreeTopicDto, indent: usize) {
    println!("{}{}", "  ".repeat(indent), topic.title);
    if let Some(children) = &topic.children {
        for child in children {
            render_tree_text(child, indent + 1);
        }
    }
}

pub(super) fn find_topic_by_path<'a>(root: &'a Topic, path: &TopicPath) -> Option<&'a Topic> {
    if path.is_root() {
        return Some(root);
    }

    let mut current = root;
    for segment in path.segments() {
        current = current
            .children
            .iter()
            .find(|child| child.title == segment.as_str())?;
    }

    Some(current)
}

pub(super) struct ResolvedTopic<'a> {
    pub(super) topic: &'a Topic,
    pub(super) path: TopicPath,
}

pub(super) enum ResolveOne<'a> {
    Found(ResolvedTopic<'a>),
    NotFound,
    Ambiguous(Vec<ResolvedTopic<'a>>),
}

pub(super) fn resolve_topic<'a>(root: &'a Topic, selector: &Selector) -> ResolveOne<'a> {
    match selector {
        Selector::Root => ResolveOne::Found(ResolvedTopic {
            topic: root,
            path: TopicPath::root(),
        }),
        Selector::Path(path) => match find_topic_by_path(root, path) {
            Some(topic) => ResolveOne::Found(ResolvedTopic {
                topic,
                path: path.clone(),
            }),
            None => ResolveOne::NotFound,
        },
        Selector::Id(id) => match find_topic_by_id(root, &TopicPath::root(), id) {
            Some(resolved) => ResolveOne::Found(resolved),
            None => ResolveOne::NotFound,
        },
        Selector::Title(title) => {
            let mut matches = Vec::new();
            collect_topics_by_title(root, &TopicPath::root(), title, &mut matches);
            match matches.len() {
                0 => ResolveOne::NotFound,
                1 => ResolveOne::Found(matches.remove(0)),
                _ => ResolveOne::Ambiguous(matches),
            }
        }
        Selector::Query { expr, .. } => {
            let mut matches = Vec::new();
            collect_topics_by_query(root, &TopicPath::root(), 0, expr, &mut matches);
            match matches.len() {
                0 => ResolveOne::NotFound,
                1 => ResolveOne::Found(matches.remove(0)),
                _ => ResolveOne::Ambiguous(matches),
            }
        }
    }
}

fn find_topic_by_id<'a>(topic: &'a Topic, path: &TopicPath, id: &str) -> Option<ResolvedTopic<'a>> {
    if topic.id.0 == id {
        return Some(ResolvedTopic {
            topic,
            path: path.clone(),
        });
    }

    for child in &topic.children {
        let child_path = path.join(child.title.clone());
        if let Some(resolved) = find_topic_by_id(child, &child_path, id) {
            return Some(resolved);
        }
    }

    None
}

fn collect_topics_by_title<'a>(
    topic: &'a Topic,
    path: &TopicPath,
    title: &str,
    matches: &mut Vec<ResolvedTopic<'a>>,
) {
    if topic.title == title {
        matches.push(ResolvedTopic {
            topic,
            path: path.clone(),
        });
    }

    for child in &topic.children {
        let child_path = path.join(child.title.clone());
        collect_topics_by_title(child, &child_path, title, matches);
    }
}

fn collect_topics_by_query<'a>(
    topic: &'a Topic,
    path: &TopicPath,
    depth: usize,
    query: &QueryExpr,
    matches: &mut Vec<ResolvedTopic<'a>>,
) {
    if query.matches_topic(topic, path, depth) {
        matches.push(ResolvedTopic {
            topic,
            path: path.clone(),
        });
    }

    for child in &topic.children {
        let child_path = path.join(child.title.clone());
        collect_topics_by_query(child, &child_path, depth + 1, query, matches);
    }
}

fn collect_find_matches(
    topic: &Topic,
    path: &TopicPath,
    sheet_title: &str,
    depth: usize,
    criterion: &FindCriterion<'_>,
    matches: &mut Vec<FindMatchDto>,
) {
    if criterion.matches_topic(topic, path, depth) {
        matches.push(FindMatchDto {
            id: topic.id.0.clone(),
            path: path.to_selector_value(),
            title: topic.title.clone(),
            sheet: sheet_title.to_owned(),
            children_count: topic.children.len(),
        });
    }

    for child in &topic.children {
        let child_path = path.join(child.title.clone());
        collect_find_matches(
            child,
            &child_path,
            sheet_title,
            depth + 1,
            criterion,
            matches,
        );
    }
}

pub(in crate::app) fn collect_added_paths(
    parent_path: &TopicPath,
    tree: &TopicTreeInputDto,
) -> Vec<String> {
    let root_path = parent_path.join(tree.title.clone());
    let mut paths = vec![root_path.to_selector_value()];

    for child in &tree.children {
        paths.extend(collect_added_paths(&root_path, child));
    }

    paths
}

pub(in crate::app) fn collect_deleted_paths(topic: &Topic, path: &TopicPath) -> Vec<String> {
    let mut paths = vec![path.to_selector_value()];

    for child in &topic.children {
        paths.extend(collect_deleted_paths(
            child,
            &path.join(child.title.clone()),
        ));
    }

    paths
}

fn collect_descendant_paths(topic: &Topic, path: &TopicPath) -> Vec<String> {
    let mut paths = Vec::new();

    for child in &topic.children {
        paths.extend(collect_deleted_paths(
            child,
            &path.join(child.title.clone()),
        ));
    }

    paths
}

fn collect_copied_paths(topic: &Topic, parent_path: &TopicPath, copied_title: &str) -> Vec<String> {
    let copied_root_path = parent_path.join(copied_title.to_owned());
    let mut paths = vec![copied_root_path.to_selector_value()];

    for child in &topic.children {
        paths.extend(collect_deleted_paths(
            child,
            &copied_root_path.join(child.title.clone()),
        ));
    }

    paths
}

fn collect_promoted_paths(topic: &Topic, path: &TopicPath) -> Vec<PromotedTopicDto> {
    let parent_path = if path.segments().is_empty() {
        TopicPath::root()
    } else {
        TopicPath::from_segments(path.segments()[..path.segments().len() - 1].to_vec())
    };

    topic
        .children
        .iter()
        .map(|child| PromotedTopicDto {
            id: child.id.0.clone(),
            from_path: path.join(child.title.clone()).to_selector_value(),
            to_path: parent_path.join(child.title.clone()).to_selector_value(),
        })
        .collect()
}

fn missing_path_segments<'a>(
    root: &'a Topic,
    target_path: &TopicPath,
) -> Option<(&'a Topic, TopicPath, Vec<String>)> {
    if target_path.is_root() {
        return None;
    }

    let mut current = root;
    let mut current_path = TopicPath::root();
    let segments = target_path.segments();

    for (index, segment) in segments.iter().enumerate() {
        if let Some(child) = current
            .children
            .iter()
            .find(|child| child.title == *segment)
        {
            current = child;
            current_path = current_path.join(segment.clone());
        } else {
            return Some((current, current_path, segments[index..].to_vec()));
        }
    }

    None
}

fn render_workbook_write_error(invocation: Invocation, json: bool, error: XMindWriteError) -> i32 {
    let error = workbook_write_error_body(&invocation.workbook, error);
    render_error(invocation, json, error)
}

fn workbook_write_error_body(workbook: &Path, error: XMindWriteError) -> CliErrorBody {
    match error {
        XMindWriteError::CandidateValidationFailed(message) => CliErrorBody::new(
            ErrorCode::ValidationFailed,
            format!("Candidate workbook failed validation: {message}"),
            false,
            "The original workbook was left unchanged. Inspect the mutation and retry.",
        )
        .with_path(workbook.display().to_string()),
        error => CliErrorBody::new(
            ErrorCode::WriteFailed,
            format!("Workbook could not be written: {error}"),
            true,
            "Check write permissions and retry.",
        )
        .with_path(workbook.display().to_string()),
    }
}

fn create_mutation_backup(invocation: &Invocation) -> Result<Option<String>, BackupError> {
    if invocation.dry_run || !invocation.backup {
        return Ok(None);
    }

    create_backup(&invocation.workbook, backup_timestamp())
        .map(|backup| Some(backup.path.display().to_string()))
}

fn render_backup_error(invocation: Invocation, json: bool, error: BackupError) -> i32 {
    let error = CliErrorBody::new(
        ErrorCode::WriteFailed,
        format!("Backup could not be written: {error}"),
        true,
        "Check backup directory permissions and retry.",
    )
    .with_path(invocation.workbook.display().to_string());
    render_error(invocation, json, error)
}

pub(super) fn render_error(invocation: Invocation, json: bool, error: CliErrorBody) -> i32 {
    let exit_code = error.exit_code;

    if json {
        let envelope = CommandEnvelope::<Value> {
            ok: false,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: invocation.applied,
            result: None,
            error: Some(error),
            warnings: Vec::new(),
        };
        crate::cli::render_json_envelope(&envelope);
    } else {
        crate::cli::render_human_error(Some(&invocation.command), &error, invocation.no_color);
    }

    exit_code
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::cli::ErrorCode;
    use crate::infra::xmind::encode::XMindWriteError;

    use super::workbook_write_error_body;

    #[test]
    fn candidate_validation_write_error_maps_to_validation_failed() {
        let error = workbook_write_error_body(
            Path::new("roadmap.xmind"),
            XMindWriteError::CandidateValidationFailed("forced validation failure".to_owned()),
        );

        assert!(matches!(error.code, ErrorCode::ValidationFailed));
        assert_eq!(error.exit_code, ErrorCode::ValidationFailed.exit_code());
        assert_eq!(error.path.as_deref(), Some("roadmap.xmind"));
    }
}
