use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::cli::{CandidateDto, CliErrorBody, CommandEnvelope, ErrorCode};
use crate::domain::path::TopicPath;
use crate::domain::selector::Selector;
use crate::domain::sheet::Sheet;

use super::{
    collect_added_paths, collect_deleted_paths, find_topic_by_path, read_workbook_or_render_error,
    renamed_path, render_error, resolve_topic, select_sheet_or_render_error, Invocation,
    ResolveOne, TopicTreeInputDto,
};

#[derive(Debug, Deserialize)]
pub(super) struct PatchFileDto {
    pub(super) ops: Vec<PatchOpDto>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PatchOpDto {
    pub(super) op: String,
    pub(super) node: Option<String>,
    pub(super) parent: Option<String>,
    pub(super) title: Option<String>,
    pub(super) fields: Option<Map<String, Value>>,
    pub(super) tree: Option<TopicTreeInputDto>,
}

impl PatchOpDto {
    pub(super) fn canonical_op(&self) -> &str {
        match self.op.as_str() {
            "delete_tree" => "delete",
            "move_tree" => "move",
            "clone_tree" => "copy",
            op => op,
        }
    }
}

pub(super) fn read_patch_file(path: &Path) -> Result<PatchFileDto, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Patch file could not be read: {error}"))?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();

    match extension {
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .map_err(|error| format!("Patch file YAML is invalid: {error}")),
        "json" => serde_json::from_str(&content)
            .map_err(|error| format!("Patch file JSON is invalid: {error}")),
        _ => Err("Patch file must use .yaml, .yml, or .json.".to_owned()),
    }
}

pub(super) fn render_patch(invocation: Invocation, json: bool, ops_path: &Path) -> i32 {
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
        let op_name = op.canonical_op();
        if matches!(op_name, "assert_exists" | "assert_not_exists") {
            match render_patch_assert_operation(invocation.clone(), json, sheet, index, op_name, op)
            {
                Ok(()) => {
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "passed",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "add" {
            match plan_patch_add(invocation.clone(), json, sheet, index, op_name, op) {
                Ok(path) => {
                    diff.push(PatchDiffEventDto {
                        event: "added",
                        path,
                    });
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "set" {
            match plan_patch_set(invocation.clone(), json, sheet, index, op_name, op) {
                Ok(path) => {
                    diff.push(PatchDiffEventDto {
                        event: "updated",
                        path,
                    });
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "replace_tree" {
            match plan_patch_replace_tree(invocation.clone(), json, sheet, index, op_name, op) {
                Ok((deleted_paths, added_paths)) => {
                    diff.extend(deleted_paths.into_iter().map(|path| PatchDiffEventDto {
                        event: "deleted",
                        path,
                    }));
                    diff.extend(added_paths.into_iter().map(|path| PatchDiffEventDto {
                        event: "added",
                        path,
                    }));
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name != "add_tree" {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("Unsupported patch operation: {op_name}"),
                true,
                "Use add_tree for the current dry-run patch slice.",
            )
            .with_operation_context(index, op_name.to_owned());
            return render_error(invocation, json, error);
        }

        let Some(parent) = &op.parent else {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                "add_tree operation is missing parent.",
                true,
                "Add a parent selector like parent: path:/Q2.",
            )
            .with_operation_context(index, op_name.to_owned());
            return render_error(invocation, json, error);
        };

        let Some(tree) = &op.tree else {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                "add_tree operation is missing tree.",
                true,
                "Add a tree object with a title.",
            )
            .with_operation_context(index, op_name.to_owned());
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
                .with_operation_context(index, op_name.to_owned());
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
            .with_operation_context(index, op_name.to_owned());
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
        diff.extend(added_paths.into_iter().map(|path| PatchDiffEventDto {
            event: "added",
            path,
        }));
        operations.push(PatchOperationDto {
            index,
            op: "add_tree".to_owned(),
            status: "planned",
        });
    }

    let summary = summarize_patch_diff(&diff);
    let result = PatchDryRunResultDto {
        will_change: summary.added + summary.updated + summary.deleted + summary.moved > 0,
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

#[derive(Debug, serde::Serialize)]
struct PatchDryRunResultDto {
    will_change: bool,
    summary: PatchSummaryDto,
    operations: Vec<PatchOperationDto>,
    diff: Vec<PatchDiffEventDto>,
}

#[derive(Debug, serde::Serialize)]
struct PatchSummaryDto {
    added: usize,
    updated: usize,
    deleted: usize,
    moved: usize,
}

#[derive(Debug, serde::Serialize)]
struct PatchOperationDto {
    index: usize,
    op: String,
    status: &'static str,
}

#[derive(Debug, serde::Serialize)]
struct PatchDiffEventDto {
    event: &'static str,
    path: String,
}

fn summarize_patch_diff(diff: &[PatchDiffEventDto]) -> PatchSummaryDto {
    PatchSummaryDto {
        added: diff.iter().filter(|event| event.event == "added").count(),
        updated: diff.iter().filter(|event| event.event == "updated").count(),
        deleted: diff.iter().filter(|event| event.event == "deleted").count(),
        moved: diff.iter().filter(|event| event.event == "moved").count(),
    }
}

fn plan_patch_add(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<String, i32> {
    let Some(parent) = &op.parent else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "add operation is missing parent.",
            true,
            "Add a parent selector like parent: path:/Q2.",
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
    };

    let Some(title) = &op.title else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "add operation is missing title.",
            true,
            "Add a non-empty title for the new topic.",
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
    };
    if title.trim().is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "add operation title must not be empty.",
            true,
            "Add a non-empty title for the new topic.",
        )
        .with_operation_context(index, op_name.to_owned())
        .with_field_path("title");
        return Err(render_error(invocation, json, error));
    }

    let parent_selector = match Selector::parse(parent) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("add parent selector is invalid: {error}"),
                true,
                "Use a parent selector like path:/Q2.",
            )
            .with_operation_context(index, op_name.to_owned());
            return Err(render_error(invocation, json, error));
        }
    };
    let Selector::Path(parent_path) = &parent_selector else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "add parent must be a path: selector.",
            true,
            "Use a parent selector like path:/Q2.",
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
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
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
    }

    Ok(parent_path.join(title.trim()).to_selector_value())
}

fn plan_patch_set(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<String, i32> {
    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set operation is missing node.",
            true,
            "Add a node selector like node: path:/Q2.",
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
    };

    let Some(fields) = &op.fields else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set operation is missing fields.",
            true,
            "Add a fields object with at least one field.",
        )
        .with_operation_context(index, op_name.to_owned())
        .with_field_path("fields");
        return Err(render_error(invocation, json, error));
    };
    if fields.is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set operation fields must not be empty.",
            true,
            "Add at least one field to update.",
        )
        .with_operation_context(index, op_name.to_owned())
        .with_field_path("fields");
        return Err(render_error(invocation, json, error));
    }

    validate_patch_set_fields(invocation.clone(), json, index, op_name, fields)?;

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("set node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            )
            .with_operation_context(index, op_name.to_owned());
            return Err(render_error(invocation, json, error));
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("set selector did not match a topic: {}", selector.render()),
                true,
                "Run tree or find to rediscover the selector, then retry.",
            )
            .with_operation_context(index, op_name.to_owned())
            .with_selector(selector.render());
            return Err(render_error(invocation, json, error));
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "set selector matched multiple topics.",
                true,
                "Retry with a selector that resolves to exactly one topic.",
            )
            .with_operation_context(index, op_name.to_owned())
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
            return Err(render_error(invocation, json, error));
        }
    };

    let path = fields
        .get("title")
        .and_then(|title| title.as_str())
        .map(|title| renamed_path(&resolved.path, title))
        .unwrap_or_else(|| resolved.path.to_selector_value());
    Ok(path)
}

fn plan_patch_replace_tree(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<(Vec<String>, Vec<String>), i32> {
    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "replace_tree operation is missing node.",
            true,
            "Add a node selector like node: path:/Q2/Payment.",
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
    };

    let Some(tree) = &op.tree else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "replace_tree operation is missing tree.",
            true,
            "Add a tree object with a title.",
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
    };
    if tree.title.trim().is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "replace_tree operation tree title must not be empty.",
            true,
            "Add a non-empty title for the replacement tree root.",
        )
        .with_operation_context(index, op_name.to_owned())
        .with_field_path("tree.title");
        return Err(render_error(invocation, json, error));
    }

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("replace_tree node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            )
            .with_operation_context(index, op_name.to_owned());
            return Err(render_error(invocation, json, error));
        }
    };

    let resolved = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "replace_tree selector did not match a topic: {}",
                    selector.render()
                ),
                true,
                "Run tree or find to rediscover the selector, then retry.",
            )
            .with_operation_context(index, op_name.to_owned())
            .with_selector(selector.render());
            return Err(render_error(invocation, json, error));
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "replace_tree selector matched multiple topics.",
                true,
                "Retry with a selector that resolves to exactly one topic.",
            )
            .with_operation_context(index, op_name.to_owned())
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
            return Err(render_error(invocation, json, error));
        }
    };
    if resolved.path.is_root() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "replace_tree cannot target the root topic.",
            true,
            "Target a non-root topic such as path:/Q2/Payment.",
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
    }

    let parent_path = TopicPath::from_segments(
        resolved.path.segments()[..resolved.path.segments().len().saturating_sub(1)].to_vec(),
    );
    let deleted_paths = collect_deleted_paths(resolved.topic, &resolved.path);
    let added_paths = collect_added_paths(&parent_path, tree);

    Ok((deleted_paths, added_paths))
}

fn validate_patch_set_fields(
    invocation: Invocation,
    json: bool,
    index: usize,
    op_name: &str,
    fields: &serde_json::Map<String, Value>,
) -> Result<(), i32> {
    for (field, value) in fields {
        let valid = match field.as_str() {
            "title" => value.as_str().is_some_and(|title| !title.trim().is_empty()),
            "note" | "hyperlink" => value.is_null() || value.is_string(),
            "labels" | "markers" => {
                value.is_null()
                    || value
                        .as_array()
                        .is_some_and(|items| items.iter().all(|item| item.is_string()))
            }
            "image" => value.is_null() || value.is_object(),
            _ => false,
        };
        if !valid {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("set operation field is invalid: {field}"),
                true,
                "Use supported fields with values of the documented type.",
            )
            .with_operation_context(index, op_name.to_owned())
            .with_field_path(format!("fields.{field}"));
            return Err(render_error(invocation, json, error));
        }
    }

    Ok(())
}

fn render_patch_assert_operation(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<(), i32> {
    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            format!("{op_name} operation is missing node."),
            true,
            "Add a node selector like node: path:/Q2.",
        )
        .with_operation_context(index, op_name.to_owned());
        return Err(render_error(invocation, json, error));
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("{op_name} node selector is invalid: {error}"),
                true,
                "Use a valid selector such as root, id:<topic-id>, path:/Q2, or title:Payment.",
            )
            .with_operation_context(index, op_name.to_owned());
            return Err(render_error(invocation, json, error));
        }
    };

    match (op_name, resolve_topic(&sheet.root, &selector)) {
        ("assert_exists", ResolveOne::Found(_)) => Ok(()),
        ("assert_exists", ResolveOne::NotFound) => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "assert_exists selector did not match a topic: {}",
                    selector.render()
                ),
                true,
                "Run tree or find to rediscover the selector, then retry.",
            )
            .with_operation_context(index, op_name.to_owned())
            .with_selector(selector.render());
            Err(render_error(invocation, json, error))
        }
        ("assert_exists", ResolveOne::Ambiguous(candidates)) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "assert_exists selector matched multiple topics.",
                true,
                "Retry with a selector that resolves to exactly one topic.",
            )
            .with_operation_context(index, op_name.to_owned())
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
            Err(render_error(invocation, json, error))
        }
        ("assert_not_exists", ResolveOne::NotFound) => Ok(()),
        ("assert_not_exists", ResolveOne::Found(_) | ResolveOne::Ambiguous(_)) => {
            let error = CliErrorBody::new(
                ErrorCode::PatchConflict,
                format!(
                    "assert_not_exists selector matched an existing topic: {}",
                    selector.render()
                ),
                false,
                "Remove or update the conflicting topic, or revise the patch precondition.",
            )
            .with_operation_context(index, op_name.to_owned())
            .with_selector(selector.render());
            Err(render_error(invocation, json, error))
        }
        _ => Ok(()),
    }
}
