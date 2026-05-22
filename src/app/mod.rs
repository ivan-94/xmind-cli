use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli::{
    CandidateDto, Cli, CliErrorBody, Command, CommandEnvelope, ErrorCode, OutputFormat,
};
use crate::domain::diff::{Diff, DiffEvent, FieldChange};
use crate::domain::path::TopicPath;
use crate::domain::query::QueryExpr;
use crate::domain::selector::Selector;
use crate::domain::sheet::Sheet;
use crate::domain::topic::Topic;
use crate::render::diff::render_human_outline;

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
        Action::Patch { ref ops } => {
            let ops = ops.clone();
            render_patch(invocation, json, &ops)
        }
        Action::Add {
            ref parent,
            ref title,
        } => {
            let parent = parent.clone();
            let title = title.clone();
            render_add(invocation, json, &parent, &title)
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
        Action::Delete { ref node } => {
            let node = node.clone();
            render_delete(invocation, json, &node)
        }
        Action::Move {
            ref node,
            ref destination,
        } => {
            let node = node.clone();
            let destination = destination.clone();
            render_move(invocation, json, &node, &destination)
        }
        Action::Noop => 0,
    }
}

#[derive(Clone)]
struct Invocation {
    command: String,
    workbook: std::path::PathBuf,
    dry_run: bool,
    applied: bool,
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
    Delete {
        node: String,
    },
    Move {
        node: String,
        destination: String,
    },
    Validate {
        strict: bool,
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
            Command::Backup(command) => {
                Some(Self::read("backup", command.workbook, sheet_selection))
            }
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
                    command.mode.dry_run,
                    sheet_selection,
                    quiet,
                )
                .with_action(Action::Add {
                    parent: command.parent,
                    title: command.title,
                }),
            ),
            Command::AddTree(command) => Some(Self::mutation(
                "add-tree",
                command.workbook,
                command.mode.dry_run,
                sheet_selection,
                quiet,
            )),
            Command::Set(command) => Some(
                Self::mutation(
                    "set",
                    command.workbook,
                    command.mode.dry_run,
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
                } else {
                    Action::Noop
                }),
            ),
            Command::Delete(command) => Some(
                Self::mutation(
                    "delete",
                    command.workbook,
                    command.mode.dry_run,
                    sheet_selection,
                    quiet,
                )
                .with_action(Action::Delete { node: command.node }),
            ),
            Command::Move(command) => Some(
                Self::mutation(
                    "move",
                    command.workbook,
                    command.mode.dry_run,
                    sheet_selection,
                    quiet,
                )
                .with_action(Action::Move {
                    node: command.node,
                    destination: command.to,
                }),
            ),
            Command::Copy(command) => Some(Self::mutation(
                "copy",
                command.workbook,
                command.mode.dry_run,
                sheet_selection,
                quiet,
            )),
            Command::Patch(command) => Some(Self::patch(
                command.workbook,
                command.mode.dry_run,
                command.ops,
                sheet_selection,
                quiet,
            )),
            Command::Import(command) => Some(Self::mutation(
                "import",
                command.workbook,
                command.mode.dry_run,
                sheet_selection,
                quiet,
            )),
            Command::Restore(command) => Some(Self::mutation(
                "restore",
                command.workbook,
                command.mode.dry_run,
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
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Self {
        Self {
            command: command.to_owned(),
            workbook,
            dry_run,
            applied: false,
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
        ops: std::path::PathBuf,
        sheet_selection: SheetSelection,
        quiet: bool,
    ) -> Self {
        Self {
            command: "patch".to_owned(),
            workbook,
            dry_run,
            applied: false,
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

fn render_add(invocation: Invocation, json: bool, parent: &str, title: &str) -> i32 {
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

    let new_topic_id = generated_topic_id(title);
    let created_topic_path = parent.path.join(title.to_owned());
    let created_path = created_topic_path.to_selector_value();
    let human_diff = Diff::from_events(vec![DiffEvent::Added {
        path: created_topic_path,
    }]);
    let result = AddDryRunResultDto {
        will_change: true,
        parent: TopicRefDto {
            id: parent.topic.id.0.clone(),
            path: parent.path.to_selector_value(),
            title: parent.topic.title.clone(),
        },
        created: CreatedTopicDto {
            path: created_path.clone(),
            title: title.to_owned(),
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
    };

    if !invocation.dry_run {
        if let Err(error) = crate::infra::xmind::encode::append_child_topic(
            &invocation.workbook,
            &parent.topic.id.0,
            title,
            &new_topic_id,
        ) {
            let error = CliErrorBody::new(
                ErrorCode::WriteFailed,
                format!("Workbook could not be written: {error}"),
                true,
                "Check write permissions and retry.",
            )
            .with_path(invocation.workbook.display().to_string());
            return render_error(invocation, json, error);
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

fn render_delete(invocation: Invocation, json: bool, node: &str) -> i32 {
    if !invocation.dry_run {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "Only delete --dry-run is implemented in this slice.",
            true,
            "Retry with --dry-run, or wait for the delete apply slice.",
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
            "Use a non-root node selector, or use --children-only when that slice is available.",
        )
        .with_selector(selector.render());
        return render_error(invocation, json, error);
    }

    let deleted = collect_deleted_paths(resolved.topic, &resolved.path);
    let diff = deleted
        .iter()
        .cloned()
        .map(|path| DiffEventDto {
            event: "deleted",
            path,
        })
        .collect::<Vec<_>>();
    let result = DeleteDryRunResultDto {
        will_change: !deleted.is_empty(),
        deleted,
        summary: SummaryDto {
            added: 0,
            updated: 0,
            deleted: diff.len(),
            moved: 0,
        },
        diff,
    };

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: false,
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

fn render_move(invocation: Invocation, json: bool, node: &str, destination: &str) -> i32 {
    if !invocation.dry_run {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "Only move --dry-run is implemented in this slice.",
            true,
            "Retry with --dry-run, or wait for the move apply slice.",
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

    let from_path = source.path.to_selector_value();
    let to_path = destination
        .path
        .join(source.topic.title.clone())
        .to_selector_value();
    let result = MoveDryRunResultDto {
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
    };

    if json {
        let envelope = CommandEnvelope {
            ok: true,
            command: Some(invocation.command),
            workbook: Some(invocation.workbook.display().to_string()),
            dry_run: invocation.dry_run,
            applied: false,
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
    let result = SetTitleDryRunResultDto {
        will_change: resolved.topic.title != title,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: None,
            old_path: Some(old_path),
            new_path: Some(new_path.clone()),
            new_note: None,
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
    };

    if !invocation.dry_run {
        if let Err(error) = crate::infra::xmind::encode::rename_topic(
            &invocation.workbook,
            &resolved.topic.id.0,
            title,
        ) {
            let error = CliErrorBody::new(
                ErrorCode::WriteFailed,
                format!("Workbook could not be written: {error}"),
                true,
                "Check write permissions and retry.",
            )
            .with_path(invocation.workbook.display().to_string());
            return render_error(invocation, json, error);
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

fn renamed_path(path: &TopicPath, title: &str) -> String {
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
    let result = SetTitleDryRunResultDto {
        will_change: true,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: Some(path.clone()),
            old_path: None,
            new_path: None,
            new_note: None,
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
    };

    if !invocation.dry_run {
        if let Err(error) = crate::infra::xmind::encode::set_topic_note(
            &invocation.workbook,
            &resolved.topic.id.0,
            note,
        ) {
            let error = CliErrorBody::new(
                ErrorCode::WriteFailed,
                format!("Workbook could not be written: {error}"),
                true,
                "Check write permissions and retry.",
            )
            .with_path(invocation.workbook.display().to_string());
            return render_error(invocation, json, error);
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
    let result = SetTitleDryRunResultDto {
        will_change: true,
        updated: UpdatedTopicDto {
            id: resolved.topic.id.0.clone(),
            path: Some(path.clone()),
            old_path: None,
            new_path: None,
            new_note: Some(new_note.clone()),
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
    };

    if !invocation.dry_run {
        if let Err(error) = crate::infra::xmind::encode::set_topic_note(
            &invocation.workbook,
            &resolved.topic.id.0,
            &new_note,
        ) {
            let error = CliErrorBody::new(
                ErrorCode::WriteFailed,
                format!("Workbook could not be written: {error}"),
                true,
                "Check write permissions and retry.",
            )
            .with_path(invocation.workbook.display().to_string());
            return render_error(invocation, json, error);
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

fn render_patch(invocation: Invocation, json: bool, ops_path: &Path) -> i32 {
    let workbook = match read_workbook_or_render_error(&invocation, json) {
        Ok(workbook) => workbook,
        Err(exit_code) => return exit_code,
    };

    if !invocation.dry_run {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "Only patch --dry-run is implemented in this slice.",
            true,
            "Retry with --dry-run, or wait for the transactional writer slice before using --apply.",
        );
        return render_error(invocation, json, error);
    }

    let patch = match read_patch_file(ops_path) {
        Ok(patch) => patch,
        Err(message) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                message,
                true,
                "Fix the patch file and retry.",
            )
            .with_path(ops_path.display().to_string());
            return render_error(invocation, json, error);
        }
    };

    let sheet = match select_sheet_or_render_error(&workbook, &invocation, json) {
        Ok(sheet) => sheet,
        Err(exit_code) => return exit_code,
    };

    let mut operations = Vec::new();
    let mut diff = Vec::new();

    for (index, op) in patch.ops.iter().enumerate() {
        if op.op != "add_tree" {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("Unsupported patch operation: {}", op.op),
                true,
                "Use add_tree for the current dry-run patch slice.",
            )
            .with_operation_context(index, op.op.clone());
            return render_error(invocation, json, error);
        }

        let Some(parent) = &op.parent else {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                "add_tree operation is missing parent.",
                true,
                "Add a parent selector like parent: path:/Q2.",
            )
            .with_operation_context(index, op.op.clone());
            return render_error(invocation, json, error);
        };

        let Some(tree) = &op.tree else {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                "add_tree operation is missing tree.",
                true,
                "Add a tree object with a title.",
            )
            .with_operation_context(index, op.op.clone());
            return render_error(invocation, json, error);
        };

        let parent_selector = match Selector::parse(parent) {
            Ok(selector) => selector,
            Err(error) => {
                let error = CliErrorBody::new(
                    ErrorCode::InvalidPatch,
                    format!("add_tree parent selector is invalid: {error}"),
                    true,
                    "Use a parent selector like path:/Q2.",
                )
                .with_operation_context(index, op.op.clone());
                return render_error(invocation, json, error);
            }
        };
        let Selector::Path(parent_path) = &parent_selector else {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                "add_tree parent must be a path: selector.",
                true,
                "Use a parent selector like path:/Q2.",
            )
            .with_operation_context(index, op.op.clone());
            return render_error(invocation, json, error);
        };

        if find_topic_by_path(&sheet.root, parent_path).is_none() {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "Parent selector did not match a topic: {}",
                    parent_selector.render()
                ),
                true,
                "Run tree or find to rediscover the parent path, then retry.",
            );
            return render_error(invocation, json, error);
        }

        let added_paths = collect_added_paths(parent_path, tree);
        diff.extend(added_paths.into_iter().map(|path| DiffEventDto {
            event: "added",
            path,
        }));
        operations.push(PatchOperationDto {
            index,
            op: "add_tree",
            status: "planned",
        });
    }

    let summary = SummaryDto {
        added: diff.len(),
        updated: 0,
        deleted: 0,
        moved: 0,
    };
    let result = PatchDryRunResultDto {
        will_change: summary.added > 0,
        summary,
        operations,
        diff,
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

fn read_workbook_or_render_error(
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

fn select_sheet_or_render_error<'a>(
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

#[derive(Debug, Deserialize)]
struct PatchFileDto {
    ops: Vec<PatchOpDto>,
}

#[derive(Debug, Deserialize)]
struct PatchOpDto {
    op: String,
    parent: Option<String>,
    tree: Option<TopicTreeInputDto>,
}

#[derive(Debug, Deserialize)]
struct TopicTreeInputDto {
    title: String,

    #[serde(default)]
    children: Vec<TopicTreeInputDto>,
}

#[derive(Debug, Serialize)]
struct PatchDryRunResultDto {
    will_change: bool,
    summary: SummaryDto,
    operations: Vec<PatchOperationDto>,
    diff: Vec<DiffEventDto>,
}

#[derive(Debug, Serialize)]
struct AddDryRunResultDto {
    will_change: bool,
    parent: TopicRefDto,
    created: CreatedTopicDto,
    summary: SummaryDto,
    diff: Vec<DiffEventDto>,
}

#[derive(Debug, Serialize)]
struct DeleteDryRunResultDto {
    will_change: bool,
    deleted: Vec<String>,
    summary: SummaryDto,
    diff: Vec<DiffEventDto>,
}

#[derive(Debug, Serialize)]
struct MoveDryRunResultDto {
    will_change: bool,
    moved: MovedTopicDto,
    summary: SummaryDto,
    diff: Vec<MoveDiffEventDto>,
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
struct SetTitleDryRunResultDto {
    will_change: bool,
    updated: UpdatedTopicDto,
    summary: SummaryDto,
    diff: Vec<DiffEventDto>,
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
struct PatchOperationDto {
    index: usize,
    op: &'static str,
    status: &'static str,
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

fn read_patch_file(path: &Path) -> Result<PatchFileDto, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Patch file could not be read: {error}"))?;
    serde_yaml::from_str(&content)
        .map_err(|error| format!("Patch file could not be decoded: {error}"))
}

fn find_topic_by_path<'a>(root: &'a Topic, path: &TopicPath) -> Option<&'a Topic> {
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

struct ResolvedTopic<'a> {
    topic: &'a Topic,
    path: TopicPath,
}

enum ResolveOne<'a> {
    Found(ResolvedTopic<'a>),
    NotFound,
    Ambiguous(Vec<ResolvedTopic<'a>>),
}

fn resolve_topic<'a>(root: &'a Topic, selector: &Selector) -> ResolveOne<'a> {
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

fn collect_added_paths(parent_path: &TopicPath, tree: &TopicTreeInputDto) -> Vec<String> {
    let root_path = parent_path.join(tree.title.clone());
    let mut paths = vec![root_path.to_selector_value()];

    for child in &tree.children {
        paths.extend(collect_added_paths(&root_path, child));
    }

    paths
}

fn collect_deleted_paths(topic: &Topic, path: &TopicPath) -> Vec<String> {
    let mut paths = vec![path.to_selector_value()];

    for child in &topic.children {
        paths.extend(collect_deleted_paths(
            child,
            &path.join(child.title.clone()),
        ));
    }

    paths
}

fn render_error(invocation: Invocation, json: bool, error: CliErrorBody) -> i32 {
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
