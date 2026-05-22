use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::cli::{CandidateDto, CliErrorBody, CommandEnvelope, ErrorCode};
use crate::domain::path::TopicPath;
use crate::domain::selector::Selector;
use crate::domain::sheet::{Sheet, SheetId};
use crate::domain::topic::{AssetId, Topic, TopicId, TopicImageRef};

use super::{
    collect_added_paths, collect_copied_paths, collect_deleted_paths, collect_descendant_paths,
    find_topic_by_path, insert_position_from_spec, parse_insert_position,
    read_workbook_or_render_error, renamed_path, render_error, resolve_topic,
    select_sheet_or_render_error, Invocation, ResolveOne, TopicTreeInputDto,
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
    pub(super) target: Option<String>,
    pub(super) to: Option<String>,
    pub(super) position: Option<String>,
    pub(super) path: Option<String>,
    pub(super) title: Option<String>,
    pub(super) fields: Option<Map<String, Value>>,
    pub(super) tree: Option<TopicTreeInputDto>,
    pub(super) match_by: Option<String>,
    pub(super) prune: Option<bool>,
    pub(super) children_only: Option<bool>,
    pub(super) promote_children: Option<bool>,
    pub(super) preserve_ids: Option<bool>,
    pub(super) by: Option<String>,
    pub(super) order: Option<String>,
    pub(super) recursive: Option<bool>,
    pub(super) add_labels: Option<Vec<String>>,
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
    let mut working_sheet = clone_sheet_for_patch(sheet);

    let mut operations = Vec::new();
    let mut diff = Vec::new();
    let mut deleted_path_refs = Vec::new();

    for (index, op) in patch.ops.iter().enumerate() {
        let op_name = op.canonical_op();
        if let Err(exit_code) = reject_deleted_path_references(
            invocation.clone(),
            json,
            index,
            op_name,
            op,
            &deleted_path_refs,
        ) {
            return exit_code;
        }

        if matches!(op_name, "assert_exists" | "assert_not_exists") {
            match render_patch_assert_operation(
                invocation.clone(),
                json,
                &working_sheet,
                index,
                op_name,
                op,
            ) {
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
            match plan_patch_add(invocation.clone(), json, &working_sheet, index, op_name, op) {
                Ok(path) => {
                    diff.push(PatchDiffEventDto::path_event("added", path));
                    apply_patch_add_to_working_root(&mut working_sheet.root, op);
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
            match plan_patch_set(invocation.clone(), json, &working_sheet, index, op_name, op) {
                Ok(path) => {
                    if let Some(path) = path {
                        diff.push(PatchDiffEventDto::path_event("updated", path));
                    }
                    apply_patch_set_to_working_root(&mut working_sheet.root, op);
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
            match plan_patch_replace_tree(
                invocation.clone(),
                json,
                &working_sheet,
                index,
                op_name,
                op,
            ) {
                Ok((deleted_paths, added_paths)) => {
                    diff.extend(
                        deleted_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("deleted", path)),
                    );
                    diff.extend(
                        added_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("added", path)),
                    );
                    apply_patch_replace_tree_to_working_root(&mut working_sheet.root, op);
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "merge_tree" {
            match plan_patch_merge_tree(
                invocation.clone(),
                json,
                &working_sheet,
                index,
                op_name,
                op,
            ) {
                Ok(plan) => {
                    diff.extend(
                        plan.updated_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("updated", path)),
                    );
                    diff.extend(
                        plan.added_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("added", path)),
                    );
                    remember_deleted_paths(&mut deleted_path_refs, &plan.deleted_paths);
                    diff.extend(
                        plan.deleted_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("deleted", path)),
                    );
                    apply_patch_merge_tree_to_working_root(&mut working_sheet.root, op);
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "delete" {
            match plan_patch_delete(invocation.clone(), json, &working_sheet, index, op_name, op) {
                Ok(plan) => {
                    remember_deleted_paths(&mut deleted_path_refs, &plan.deleted_paths);
                    diff.extend(
                        plan.deleted_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("deleted", path)),
                    );
                    diff.extend(
                        plan.moved_paths
                            .into_iter()
                            .map(|(from, to)| PatchDiffEventDto::moved(from, to)),
                    );
                    apply_patch_delete_to_working_root(&mut working_sheet.root, op);
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "move" {
            match plan_patch_move(invocation.clone(), json, &working_sheet, index, op_name, op) {
                Ok((from, to)) => {
                    diff.push(PatchDiffEventDto::moved(from, to));
                    apply_patch_move_to_working_root(&mut working_sheet.root, op);
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "copy" {
            match plan_patch_copy(invocation.clone(), json, &working_sheet, index, op_name, op) {
                Ok(added_paths) => {
                    diff.extend(
                        added_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("added", path)),
                    );
                    apply_patch_copy_to_working_root(&mut working_sheet.root, op);
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "ensure_path" {
            match plan_patch_ensure_path(
                invocation.clone(),
                json,
                &working_sheet,
                index,
                op_name,
                op,
            ) {
                Ok(added_paths) => {
                    diff.extend(
                        added_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("added", path)),
                    );
                    apply_patch_ensure_path_to_working_root(&mut working_sheet.root, op);
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "sort_children" {
            match plan_patch_sort_children(
                invocation.clone(),
                json,
                &working_sheet,
                index,
                op_name,
                op,
            ) {
                Ok(updated_path) => {
                    if let Some(path) = updated_path {
                        diff.push(PatchDiffEventDto::path_event("updated", path));
                    }
                    apply_patch_sort_children_to_working_root(&mut working_sheet.root, op);
                    operations.push(PatchOperationDto {
                        index,
                        op: op_name.to_owned(),
                        status: "planned",
                    });
                    continue;
                }
                Err(exit_code) => return exit_code,
            }
        } else if op_name == "set_tree_metadata" {
            match plan_patch_set_tree_metadata(
                invocation.clone(),
                json,
                &working_sheet,
                index,
                op_name,
                op,
            ) {
                Ok(updated_paths) => {
                    diff.extend(
                        updated_paths
                            .into_iter()
                            .map(|path| PatchDiffEventDto::path_event("updated", path)),
                    );
                    apply_patch_set_tree_metadata_to_working_root(&mut working_sheet.root, op);
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
            .with_operation_field_context(index, op_name.to_owned(), "parent");
            return render_error(invocation, json, error);
        };

        let Some(tree) = &op.tree else {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                "add_tree operation is missing tree.",
                true,
                "Add a tree object with a title.",
            )
            .with_operation_field_context(index, op_name.to_owned(), "tree");
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

        if find_topic_by_path(&working_sheet.root, parent_path).is_none() {
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
        diff.extend(
            added_paths
                .into_iter()
                .map(|path| PatchDiffEventDto::path_event("added", path)),
        );
        apply_patch_add_tree_to_working_root(&mut working_sheet.root, op);
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
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<String>,
}

impl PatchDiffEventDto {
    fn path_event(event: &'static str, path: String) -> Self {
        Self {
            event,
            path: Some(path),
            from: None,
            to: None,
        }
    }

    fn moved(from: String, to: String) -> Self {
        Self {
            event: "moved",
            path: None,
            from: Some(from),
            to: Some(to),
        }
    }
}

fn summarize_patch_diff(diff: &[PatchDiffEventDto]) -> PatchSummaryDto {
    PatchSummaryDto {
        added: diff.iter().filter(|event| event.event == "added").count(),
        updated: diff.iter().filter(|event| event.event == "updated").count(),
        deleted: diff.iter().filter(|event| event.event == "deleted").count(),
        moved: diff.iter().filter(|event| event.event == "moved").count(),
    }
}

fn remember_deleted_paths(deleted_path_refs: &mut Vec<TopicPath>, deleted_paths: &[String]) {
    deleted_path_refs.extend(
        deleted_paths
            .iter()
            .filter_map(|path| TopicPath::parse_selector_value(path).ok()),
    );
}

fn reject_deleted_path_references(
    invocation: Invocation,
    json: bool,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
    deleted_path_refs: &[TopicPath],
) -> Result<(), i32> {
    if deleted_path_refs.is_empty() {
        return Ok(());
    }

    for (field, selector_value) in [
        ("node", op.node.as_deref()),
        ("target", op.target.as_deref()),
        ("parent", op.parent.as_deref()),
        ("to", op.to.as_deref()),
    ] {
        let Some(selector_value) = selector_value else {
            continue;
        };
        let Ok(Selector::Path(path)) = Selector::parse(selector_value) else {
            continue;
        };
        if deleted_path_refs
            .iter()
            .any(|deleted_path| topic_path_is_same_or_descendant(&path, deleted_path))
        {
            let selector = Selector::Path(path).render();
            let error = CliErrorBody::new(
                ErrorCode::PatchConflict,
                format!("Patch operation references a topic deleted by an earlier operation: {selector}"),
                false,
                "Remove the later operation, or revise the earlier delete/prune operation so this path is preserved.",
            )
            .with_operation_field_context(index, op_name.to_owned(), field)
            .with_selector(selector);
            return Err(render_error(invocation, json, error));
        }
    }

    Ok(())
}

fn topic_path_is_same_or_descendant(path: &TopicPath, ancestor: &TopicPath) -> bool {
    path.segments().starts_with(ancestor.segments())
}

fn clone_sheet_for_patch(sheet: &Sheet) -> Sheet {
    Sheet {
        id: SheetId(sheet.id.0.clone()),
        title: sheet.title.clone(),
        root: clone_topic_for_patch(&sheet.root),
    }
}

fn clone_topic_for_patch(topic: &Topic) -> Topic {
    Topic {
        id: TopicId(topic.id.0.clone()),
        title: topic.title.clone(),
        note: topic.note.clone(),
        labels: topic.labels.clone(),
        markers: topic.markers.clone(),
        hyperlink: topic.hyperlink.clone(),
        image: topic.image.as_ref().map(|image| TopicImageRef {
            asset_id: image.asset_id.clone(),
            alt: image.alt.clone(),
            title: image.title.clone(),
        }),
        children: topic.children.iter().map(clone_topic_for_patch).collect(),
    }
}

fn apply_patch_add_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let (Some(parent), Some(title)) = (op.parent.as_deref(), op.title.as_deref()) else {
        return;
    };
    let Ok(Selector::Path(parent_path)) = Selector::parse(parent) else {
        return;
    };
    if let Some(parent_topic) = find_topic_by_path_mut(root, &parent_path) {
        parent_topic.children.push(new_working_topic(title));
    }
}

fn apply_patch_add_tree_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let (Some(parent), Some(tree)) = (op.parent.as_deref(), op.tree.as_ref()) else {
        return;
    };
    let Ok(Selector::Path(parent_path)) = Selector::parse(parent) else {
        return;
    };
    if let Some(parent_topic) = find_topic_by_path_mut(root, &parent_path) {
        parent_topic.children.push(topic_from_tree_input(tree));
    }
}

fn apply_patch_ensure_path_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let Some(path) = op.path.as_deref() else {
        return;
    };
    let Ok(path) = TopicPath::parse_selector_value(path) else {
        return;
    };
    let mut current = root;
    for segment in path.segments() {
        let child_index = current
            .children
            .iter()
            .position(|child| child.title == *segment)
            .unwrap_or_else(|| {
                current.children.push(new_working_topic(segment));
                current.children.len() - 1
            });
        current = &mut current.children[child_index];
    }
}

fn apply_patch_set_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let (Some(node), Some(fields)) = (op.node.as_deref(), op.fields.as_ref()) else {
        return;
    };
    let Ok(Selector::Path(path)) = Selector::parse(node) else {
        return;
    };
    let Some(topic) = find_topic_by_path_mut(root, &path) else {
        return;
    };
    for (field, value) in fields {
        match field.as_str() {
            "title" => {
                if let Some(title) = value.as_str() {
                    topic.title = title.to_owned();
                }
            }
            "note" => {
                topic.note = value.as_str().map(str::to_owned);
            }
            "labels" => {
                topic.labels = value
                    .as_array()
                    .map(|labels| {
                        labels
                            .iter()
                            .filter_map(|label| label.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default();
            }
            "markers" => {
                topic.markers = value
                    .as_array()
                    .map(|markers| {
                        markers
                            .iter()
                            .filter_map(|marker| marker.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default();
            }
            "hyperlink" => {
                topic.hyperlink = value.as_str().map(str::to_owned);
            }
            "image" if value.is_null() => {
                topic.image = None;
            }
            _ => {}
        }
    }
}

fn apply_patch_replace_tree_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let (Some(node), Some(tree)) = (op.node.as_deref(), op.tree.as_ref()) else {
        return;
    };
    let Ok(Selector::Path(path)) = Selector::parse(node) else {
        return;
    };
    if let Some(topic) = find_topic_by_path_mut(root, &path) {
        *topic = topic_from_tree_input(tree);
    }
}

fn apply_patch_merge_tree_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let (Some(target), Some(tree)) = (op.target.as_deref(), op.tree.as_ref()) else {
        return;
    };
    let Ok(Selector::Path(path)) = Selector::parse(target) else {
        return;
    };
    let match_by = op.match_by.as_deref().unwrap_or("title_path");
    let prune = op.prune.unwrap_or(false);
    if let Some(topic) = find_topic_by_path_mut(root, &path) {
        apply_merge_tree_to_topic(topic, &path, tree, match_by, prune);
    }
}

fn apply_patch_delete_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let Some(node) = op.node.as_deref() else {
        return;
    };
    let Ok(Selector::Path(path)) = Selector::parse(node) else {
        return;
    };
    if op.children_only.unwrap_or(false) {
        if let Some(topic) = find_topic_by_path_mut(root, &path) {
            topic.children.clear();
        }
    } else if op.promote_children.unwrap_or(false) {
        promote_children_at_path(root, &path);
    } else {
        remove_topic_by_path(root, &path);
    }
}

fn apply_patch_move_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let (Some(node), Some(destination)) = (op.node.as_deref(), op.to.as_deref()) else {
        return;
    };
    let (Ok(Selector::Path(source_path)), Ok(Selector::Path(destination_path))) =
        (Selector::parse(node), Selector::parse(destination))
    else {
        return;
    };
    let Some(topic) = remove_topic_by_path(root, &source_path) else {
        return;
    };
    if let Some(destination) = find_topic_by_path_mut(root, &destination_path) {
        destination.children.push(topic);
    }
}

fn apply_patch_copy_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let (Some(node), Some(destination)) = (op.node.as_deref(), op.to.as_deref()) else {
        return;
    };
    let (Ok(Selector::Path(source_path)), Ok(Selector::Path(destination_path))) =
        (Selector::parse(node), Selector::parse(destination))
    else {
        return;
    };
    let Some(source) = find_topic_by_path_mut(root, &source_path).map(|topic| {
        let mut topic = clone_topic_for_patch(topic);
        if let Some(title) = &op.title {
            topic.title = title.clone();
        }
        topic
    }) else {
        return;
    };
    if let Some(destination) = find_topic_by_path_mut(root, &destination_path) {
        destination.children.push(source);
    }
}

fn apply_patch_sort_children_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let Some(node) = op.node.as_deref() else {
        return;
    };
    let Ok(Selector::Path(path)) = Selector::parse(node) else {
        return;
    };
    let order = op.order.as_deref().unwrap_or("asc");
    if let Some(topic) = find_topic_by_path_mut(root, &path) {
        topic
            .children
            .sort_by(|left, right| left.title.cmp(&right.title));
        if order == "desc" {
            topic.children.reverse();
        }
    }
}

fn apply_patch_set_tree_metadata_to_working_root(root: &mut Topic, op: &PatchOpDto) {
    let (Some(node), Some(add_labels)) = (op.node.as_deref(), op.add_labels.as_ref()) else {
        return;
    };
    let Ok(Selector::Path(path)) = Selector::parse(node) else {
        return;
    };
    let recursive = op.recursive.unwrap_or(false);
    if let Some(topic) = find_topic_by_path_mut(root, &path) {
        add_labels_to_topic(topic, add_labels, recursive);
    }
}

fn apply_merge_tree_to_topic(
    topic: &mut Topic,
    path: &TopicPath,
    tree: &TopicTreeInputDto,
    match_by: &str,
    prune: bool,
) {
    topic.title = tree.title.clone();
    if let Some(note) = &tree.note {
        topic.note = Some(note.clone());
    }
    if !tree.labels.is_empty() {
        topic.labels = tree.labels.clone();
    }
    if !tree.markers.is_empty() {
        topic.markers = tree.markers.clone();
    }
    if let Some(image) = &tree.image {
        topic.image = image.asset_id.as_ref().map(|asset_id| {
            TopicImageRef::new(
                AssetId::new(asset_id.clone()),
                image.alt.clone(),
                image.title.clone(),
            )
        });
    }

    let original_child_count = topic.children.len();
    let mut matched_original_indices = Vec::new();
    for child_tree in &tree.children {
        if let Some(child_index) = find_merge_tree_child_index(topic, path, child_tree, match_by) {
            matched_original_indices.push(child_index);
            let child_path = path.join(topic.children[child_index].title.clone());
            apply_merge_tree_to_topic(
                &mut topic.children[child_index],
                &child_path,
                child_tree,
                match_by,
                prune,
            );
        } else {
            topic.children.push(topic_from_tree_input(child_tree));
        }
    }

    if prune {
        let mut index = 0;
        topic.children.retain(|_| {
            let keep = index >= original_child_count || matched_original_indices.contains(&index);
            index += 1;
            keep
        });
    }
}

fn find_merge_tree_child_index(
    topic: &Topic,
    parent_path: &TopicPath,
    child_tree: &TopicTreeInputDto,
    match_by: &str,
) -> Option<usize> {
    topic.children.iter().position(|child| {
        merge_tree_child_matches(
            child,
            &parent_path.join(child.title.clone()),
            child_tree,
            match_by,
        )
    })
}

fn promote_children_at_path(root: &mut Topic, path: &TopicPath) {
    if path.is_root() {
        return;
    }
    let Some((parent, title)) = find_parent_topic_mut(root, path) else {
        return;
    };
    let Some(index) = parent
        .children
        .iter()
        .position(|child| child.title == title)
    else {
        return;
    };
    let removed = parent.children.remove(index);
    for (offset, child) in removed.children.into_iter().enumerate() {
        parent.children.insert(index + offset, child);
    }
}

fn remove_topic_by_path(root: &mut Topic, path: &TopicPath) -> Option<Topic> {
    if path.is_root() {
        return None;
    }
    let (parent, title) = find_parent_topic_mut(root, path)?;
    let index = parent
        .children
        .iter()
        .position(|child| child.title == title)?;
    Some(parent.children.remove(index))
}

fn find_parent_topic_mut<'a>(
    root: &'a mut Topic,
    path: &TopicPath,
) -> Option<(&'a mut Topic, String)> {
    let (title, parent_segments) = path.segments().split_last()?;
    let parent_path = TopicPath::from_segments(parent_segments.to_vec());
    find_topic_by_path_mut(root, &parent_path).map(|parent| (parent, title.clone()))
}

fn add_labels_to_topic(topic: &mut Topic, labels: &[String], recursive: bool) {
    for label in labels {
        if !topic.labels.contains(label) {
            topic.labels.push(label.clone());
        }
    }
    if recursive {
        for child in &mut topic.children {
            add_labels_to_topic(child, labels, recursive);
        }
    }
}

fn find_topic_by_path_mut<'a>(topic: &'a mut Topic, path: &TopicPath) -> Option<&'a mut Topic> {
    if path.is_root() {
        return Some(topic);
    }

    let (first, rest) = path.segments().split_first()?;
    let child = topic
        .children
        .iter_mut()
        .find(|child| child.title == *first)?;
    find_topic_by_segments_mut(child, rest)
}

fn find_topic_by_segments_mut<'a>(
    topic: &'a mut Topic,
    segments: &[String],
) -> Option<&'a mut Topic> {
    let Some((first, rest)) = segments.split_first() else {
        return Some(topic);
    };
    let child = topic
        .children
        .iter_mut()
        .find(|child| child.title == *first)?;
    find_topic_by_segments_mut(child, rest)
}

fn topic_from_tree_input(tree: &TopicTreeInputDto) -> Topic {
    Topic {
        id: TopicId(
            tree.id
                .clone()
                .unwrap_or_else(|| working_topic_id(&tree.title)),
        ),
        title: tree.title.clone(),
        note: tree.note.clone(),
        labels: tree.labels.clone(),
        markers: tree.markers.clone(),
        hyperlink: None,
        image: tree.image.as_ref().and_then(|image| {
            image.asset_id.as_ref().map(|asset_id| {
                TopicImageRef::new(
                    AssetId::new(asset_id.clone()),
                    image.alt.clone(),
                    image.title.clone(),
                )
            })
        }),
        children: tree.children.iter().map(topic_from_tree_input).collect(),
    }
}

fn new_working_topic(title: &str) -> Topic {
    Topic {
        id: TopicId(working_topic_id(title)),
        title: title.to_owned(),
        note: None,
        labels: Vec::new(),
        markers: Vec::new(),
        hyperlink: None,
        image: None,
        children: Vec::new(),
    }
}

fn working_topic_id(title: &str) -> String {
    let slug = title
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();

    if slug.is_empty() {
        "topic-patch".to_owned()
    } else {
        format!("topic-{slug}")
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
        .with_operation_field_context(index, op_name.to_owned(), "parent");
        return Err(render_error(invocation, json, error));
    };

    let Some(title) = &op.title else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "add operation is missing title.",
            true,
            "Add a non-empty title for the new topic.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "title");
        return Err(render_error(invocation, json, error));
    };
    if title.trim().is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "add operation title must not be empty.",
            true,
            "Add a non-empty title for the new topic.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "title");
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
) -> Result<Option<String>, i32> {
    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set operation is missing node.",
            true,
            "Add a node selector like node: path:/Q2.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "node");
        return Err(render_error(invocation, json, error));
    };

    let Some(fields) = &op.fields else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set operation is missing fields.",
            true,
            "Add a fields object with at least one field.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "fields");
        return Err(render_error(invocation, json, error));
    };
    if fields.is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set operation fields must not be empty.",
            true,
            "Add at least one field to update.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "fields");
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

    if patch_set_changes_topic(resolved.topic, fields) {
        let path = fields
            .get("title")
            .and_then(|title| title.as_str())
            .map(|title| renamed_path(&resolved.path, title))
            .unwrap_or_else(|| resolved.path.to_selector_value());
        Ok(Some(path))
    } else {
        Ok(None)
    }
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
        .with_operation_field_context(index, op_name.to_owned(), "node");
        return Err(render_error(invocation, json, error));
    };

    let Some(tree) = &op.tree else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "replace_tree operation is missing tree.",
            true,
            "Add a tree object with a title.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "tree");
        return Err(render_error(invocation, json, error));
    };
    if tree.title.trim().is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "replace_tree operation tree title must not be empty.",
            true,
            "Add a non-empty title for the replacement tree root.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "tree.title");
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

fn plan_patch_merge_tree(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<PatchMergeTreePlan, i32> {
    let match_by = op.match_by.as_deref().unwrap_or("title_path");
    if !matches!(match_by, "title_path" | "id" | "path" | "title") {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            format!("merge_tree match_by is not implemented: {match_by}"),
            true,
            "Use match_by: title_path for this patch slice.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "match_by");
        return Err(render_error(invocation, json, error));
    }
    let Some(target) = &op.target else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "merge_tree operation is missing target.",
            true,
            "Add a target selector like target: path:/Q2/Payment.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "target");
        return Err(render_error(invocation, json, error));
    };

    let Some(tree) = &op.tree else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "merge_tree operation is missing tree.",
            true,
            "Add a tree object with a title.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "tree");
        return Err(render_error(invocation, json, error));
    };
    if tree.title.trim().is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "merge_tree operation tree title must not be empty.",
            true,
            "Add a non-empty title for the merge tree root.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "tree.title");
        return Err(render_error(invocation, json, error));
    }
    if match_by == "id" {
        if let Some(field_path) = first_tree_missing_id_path(tree, "tree") {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                "merge_tree match_by: id requires every input tree node to include an id.",
                true,
                "Add id values from a prior read/export result or use match_by: title_path.",
            )
            .with_operation_field_context(index, op_name.to_owned(), field_path);
            return Err(render_error(invocation, json, error));
        }
    }
    if match_by == "path" {
        if let Some(field_path) = first_tree_missing_path_path(tree, "tree") {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                "merge_tree match_by: path requires every input tree node to include a path.",
                true,
                "Add canonical path values from a prior read/tree result or use match_by: title_path.",
            )
            .with_operation_field_context(index, op_name.to_owned(), field_path);
            return Err(render_error(invocation, json, error));
        }
    }

    let selector = match Selector::parse(target) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("merge_tree target selector is invalid: {error}"),
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
                    "merge_tree target selector did not match a topic: {}",
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
                "merge_tree target selector matched multiple topics.",
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

    let mut diff = MergeTreeDiffAccumulator::default();
    collect_merge_tree_diff(
        resolved.topic,
        &resolved.path,
        tree,
        match_by,
        op.prune.unwrap_or(false),
        &mut diff,
    );

    Ok(PatchMergeTreePlan {
        updated_paths: diff.updated_paths,
        added_paths: diff.added_paths,
        deleted_paths: diff.deleted_paths,
    })
}

fn first_tree_missing_id_path(tree: &TopicTreeInputDto, field_path: &str) -> Option<String> {
    if tree.id.as_ref().map_or(true, |id| id.trim().is_empty()) {
        return Some(format!("{field_path}.id"));
    }

    for (index, child) in tree.children.iter().enumerate() {
        if let Some(path) =
            first_tree_missing_id_path(child, &format!("{field_path}.children[{index}]"))
        {
            return Some(path);
        }
    }

    None
}

fn first_tree_missing_path_path(tree: &TopicTreeInputDto, field_path: &str) -> Option<String> {
    if tree
        .path
        .as_ref()
        .map_or(true, |path| path.trim().is_empty())
    {
        return Some(format!("{field_path}.path"));
    }

    for (index, child) in tree.children.iter().enumerate() {
        if let Some(path) =
            first_tree_missing_path_path(child, &format!("{field_path}.children[{index}]"))
        {
            return Some(path);
        }
    }

    None
}

struct PatchMergeTreePlan {
    updated_paths: Vec<String>,
    added_paths: Vec<String>,
    deleted_paths: Vec<String>,
}

#[derive(Default)]
struct MergeTreeDiffAccumulator {
    updated_paths: Vec<String>,
    added_paths: Vec<String>,
    deleted_paths: Vec<String>,
}

fn plan_patch_delete(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<PatchDeletePlan, i32> {
    if op.children_only.unwrap_or(false) && op.promote_children.unwrap_or(false) {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "delete cannot use children_only and promote_children together.",
            true,
            "Choose one delete mode and retry.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "children_only");
        return Err(render_error(invocation, json, error));
    }

    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "delete operation is missing node.",
            true,
            "Add a node selector like node: path:/Q2/Payment.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "node");
        return Err(render_error(invocation, json, error));
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("delete node selector is invalid: {error}"),
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
                    "delete selector did not match a topic: {}",
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
                "delete selector matched multiple topics.",
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
            ErrorCode::RootOperationNotAllowed,
            "Deleting the root topic is not allowed.",
            true,
            "Use a non-root node selector.",
        )
        .with_operation_context(index, op_name.to_owned())
        .with_selector(selector.render());
        return Err(render_error(invocation, json, error));
    }

    if op.children_only.unwrap_or(false) {
        Ok(PatchDeletePlan {
            deleted_paths: collect_descendant_paths(resolved.topic, &resolved.path),
            moved_paths: Vec::new(),
        })
    } else if op.promote_children.unwrap_or(false) {
        Ok(PatchDeletePlan {
            deleted_paths: vec![resolved.path.to_selector_value()],
            moved_paths: collect_promoted_child_moves(resolved.topic, &resolved.path),
        })
    } else {
        Ok(PatchDeletePlan {
            deleted_paths: collect_deleted_paths(resolved.topic, &resolved.path),
            moved_paths: Vec::new(),
        })
    }
}

struct PatchDeletePlan {
    deleted_paths: Vec<String>,
    moved_paths: Vec<(String, String)>,
}

fn collect_promoted_child_moves(topic: &Topic, path: &TopicPath) -> Vec<(String, String)> {
    let parent_path = TopicPath::from_segments(
        path.segments()[..path.segments().len().saturating_sub(1)].to_vec(),
    );

    topic
        .children
        .iter()
        .map(|child| {
            (
                path.join(child.title.clone()).to_selector_value(),
                parent_path.join(child.title.clone()).to_selector_value(),
            )
        })
        .collect()
}

fn plan_patch_move(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<(String, String), i32> {
    let position = match parse_insert_position(op.position.clone()) {
        Ok(position) => position,
        Err(error) => {
            return Err(render_error(
                invocation,
                json,
                error.with_operation_context(index, op_name.to_owned()),
            ));
        }
    };

    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "move operation is missing node.",
            true,
            "Add a node selector like node: path:/Q2/Payment.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "node");
        return Err(render_error(invocation, json, error));
    };
    let Some(destination) = &op.to else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "move operation is missing to.",
            true,
            "Add a destination selector like to: path:/Q3.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "to");
        return Err(render_error(invocation, json, error));
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("move node selector is invalid: {error}"),
                true,
                "Use a valid selector such as id:<topic-id>, path:/Q2, or title:Payment.",
            )
            .with_operation_context(index, op_name.to_owned());
            return Err(render_error(invocation, json, error));
        }
    };
    let destination_selector = match Selector::parse(destination) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("move destination selector is invalid: {error}"),
                true,
                "Use a valid destination selector such as root, id:<topic-id>, or path:/Q2.",
            )
            .with_operation_context(index, op_name.to_owned());
            return Err(render_error(invocation, json, error));
        }
    };

    let source = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("move selector did not match a topic: {}", selector.render()),
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
                "move selector matched multiple topics.",
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
    if source.path.is_root() {
        let error = CliErrorBody::new(
            ErrorCode::RootOperationNotAllowed,
            "Moving the root topic is not allowed.",
            true,
            "Use a non-root node selector.",
        )
        .with_operation_context(index, op_name.to_owned())
        .with_selector(selector.render());
        return Err(render_error(invocation, json, error));
    }

    let destination = match resolve_topic(&sheet.root, &destination_selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "move destination selector did not match a topic: {}",
                    destination_selector.render()
                ),
                true,
                "Run tree or find to rediscover the destination selector, then retry.",
            )
            .with_operation_context(index, op_name.to_owned())
            .with_selector(destination_selector.render());
            return Err(render_error(invocation, json, error));
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "move destination selector matched multiple topics.",
                true,
                "Retry with a selector that resolves to exactly one topic.",
            )
            .with_operation_context(index, op_name.to_owned())
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
            return Err(render_error(invocation, json, error));
        }
    };

    if let Err(error) = insert_position_from_spec(position, &destination, &sheet.root) {
        return Err(render_error(
            invocation,
            json,
            error.with_operation_context(index, op_name.to_owned()),
        ));
    }

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
        .with_operation_context(index, op_name.to_owned())
        .with_selector(destination_selector.render());
        return Err(render_error(invocation, json, error));
    }

    Ok((
        source.path.to_selector_value(),
        destination
            .path
            .join(source.topic.title.clone())
            .to_selector_value(),
    ))
}

fn plan_patch_copy(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<Vec<String>, i32> {
    if op.preserve_ids.unwrap_or(false) {
        let error = CliErrorBody::new(
            ErrorCode::PatchConflict,
            "copy preserve_ids would create duplicate topic ids in the same workbook.",
            true,
            "Omit preserve_ids or set preserve_ids: false.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "preserve_ids");
        return Err(render_error(invocation, json, error));
    }

    let position = match parse_insert_position(op.position.clone()) {
        Ok(position) => position,
        Err(error) => {
            return Err(render_error(
                invocation,
                json,
                error.with_operation_context(index, op_name.to_owned()),
            ));
        }
    };

    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "copy operation is missing node.",
            true,
            "Add a node selector like node: path:/Q2/Payment.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "node");
        return Err(render_error(invocation, json, error));
    };
    let Some(destination) = &op.to else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "copy operation is missing to.",
            true,
            "Add a destination selector like to: path:/Q3.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "to");
        return Err(render_error(invocation, json, error));
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("copy node selector is invalid: {error}"),
                true,
                "Use a valid selector such as id:<topic-id>, path:/Q2, or title:Payment.",
            )
            .with_operation_context(index, op_name.to_owned());
            return Err(render_error(invocation, json, error));
        }
    };
    let destination_selector = match Selector::parse(destination) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("copy destination selector is invalid: {error}"),
                true,
                "Use a valid destination selector such as root, id:<topic-id>, or path:/Q2.",
            )
            .with_operation_context(index, op_name.to_owned());
            return Err(render_error(invocation, json, error));
        }
    };

    let source = match resolve_topic(&sheet.root, &selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!("copy selector did not match a topic: {}", selector.render()),
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
                "copy selector matched multiple topics.",
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
    if source.path.is_root() {
        let error = CliErrorBody::new(
            ErrorCode::RootOperationNotAllowed,
            "Copying the root topic is not allowed.",
            true,
            "Use a non-root node selector.",
        )
        .with_operation_context(index, op_name.to_owned())
        .with_selector(selector.render());
        return Err(render_error(invocation, json, error));
    }

    let destination = match resolve_topic(&sheet.root, &destination_selector) {
        ResolveOne::Found(resolved) => resolved,
        ResolveOne::NotFound => {
            let error = CliErrorBody::new(
                ErrorCode::NotFound,
                format!(
                    "copy destination selector did not match a topic: {}",
                    destination_selector.render()
                ),
                true,
                "Run tree or find to rediscover the destination selector, then retry.",
            )
            .with_operation_context(index, op_name.to_owned())
            .with_selector(destination_selector.render());
            return Err(render_error(invocation, json, error));
        }
        ResolveOne::Ambiguous(candidates) => {
            let error = CliErrorBody::new(
                ErrorCode::AmbiguousSelector,
                "copy destination selector matched multiple topics.",
                true,
                "Retry with a selector that resolves to exactly one topic.",
            )
            .with_operation_context(index, op_name.to_owned())
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
            return Err(render_error(invocation, json, error));
        }
    };

    if let Err(error) = insert_position_from_spec(position, &destination, &sheet.root) {
        return Err(render_error(
            invocation,
            json,
            error.with_operation_context(index, op_name.to_owned()),
        ));
    }

    let copied_title = op
        .title
        .clone()
        .unwrap_or_else(|| source.topic.title.clone());
    if copied_title.is_empty() {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "copy title cannot be empty.",
            true,
            "Omit title or provide a non-empty title.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "title");
        return Err(render_error(invocation, json, error));
    }

    Ok(collect_copied_paths(
        source.topic,
        &destination.path,
        &copied_title,
    ))
}

fn plan_patch_ensure_path(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<Vec<String>, i32> {
    let Some(path) = &op.path else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "ensure_path operation is missing path.",
            true,
            "Add a canonical path like path: /Q2/Payment.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "path");
        return Err(render_error(invocation, json, error));
    };

    let target_path = match TopicPath::parse_selector_value(path) {
        Ok(path) => path,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("ensure_path path is invalid: {error}"),
                true,
                "Use an absolute canonical path such as /Q2/Payment, without the path: prefix.",
            )
            .with_operation_field_context(index, op_name.to_owned(), "path");
            return Err(render_error(invocation, json, error));
        }
    };

    Ok(collect_missing_path_additions(&sheet.root, &target_path))
}

fn plan_patch_sort_children(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<Option<String>, i32> {
    let by = op.by.as_deref().unwrap_or("title");
    if by != "title" {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            format!("sort_children by is not supported: {by}"),
            true,
            "Use by: title.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "by");
        return Err(render_error(invocation, json, error));
    }

    let order = op.order.as_deref().unwrap_or("asc");
    if !matches!(order, "asc" | "desc") {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            format!("sort_children order is not supported: {order}"),
            true,
            "Use order: asc or order: desc.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "order");
        return Err(render_error(invocation, json, error));
    }

    if op.recursive.unwrap_or(false) {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "sort_children recursive is not implemented in this slice.",
            true,
            "Omit recursive or set recursive: false.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "recursive");
        return Err(render_error(invocation, json, error));
    }

    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "sort_children operation is missing node.",
            true,
            "Add a node selector like node: path:/Q2.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "node");
        return Err(render_error(invocation, json, error));
    };

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("sort_children node selector is invalid: {error}"),
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
                    "sort_children selector did not match a topic: {}",
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
                "sort_children selector matched multiple topics.",
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

    let current_titles = resolved
        .topic
        .children
        .iter()
        .map(|child| child.title.as_str())
        .collect::<Vec<_>>();
    let mut sorted_titles = current_titles.clone();
    sorted_titles.sort_unstable();
    if order == "desc" {
        sorted_titles.reverse();
    }

    if current_titles == sorted_titles {
        Ok(None)
    } else {
        Ok(Some(resolved.path.to_selector_value()))
    }
}

fn plan_patch_set_tree_metadata(
    invocation: Invocation,
    json: bool,
    sheet: &Sheet,
    index: usize,
    op_name: &str,
    op: &PatchOpDto,
) -> Result<Vec<String>, i32> {
    let Some(node) = &op.node else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set_tree_metadata operation is missing node.",
            true,
            "Add a node selector like node: path:/Q2.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "node");
        return Err(render_error(invocation, json, error));
    };

    let Some(add_labels) = &op.add_labels else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set_tree_metadata operation is missing add_labels.",
            true,
            "Add add_labels with one or more label values.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "add_labels");
        return Err(render_error(invocation, json, error));
    };
    if add_labels.is_empty() || add_labels.iter().any(|label| label.trim().is_empty()) {
        let error = CliErrorBody::new(
            ErrorCode::InvalidPatch,
            "set_tree_metadata add_labels must contain non-empty labels.",
            true,
            "Remove empty labels and retry.",
        )
        .with_operation_field_context(index, op_name.to_owned(), "add_labels");
        return Err(render_error(invocation, json, error));
    }

    let selector = match Selector::parse(node) {
        Ok(selector) => selector,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidPatch,
                format!("set_tree_metadata node selector is invalid: {error}"),
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
                    "set_tree_metadata selector did not match a topic: {}",
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
                "set_tree_metadata selector matched multiple topics.",
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

    let mut updated_paths = Vec::new();
    collect_set_tree_metadata_updates(
        resolved.topic,
        &resolved.path,
        add_labels,
        op.recursive.unwrap_or(false),
        &mut updated_paths,
    );
    Ok(updated_paths)
}

fn collect_set_tree_metadata_updates(
    topic: &Topic,
    path: &TopicPath,
    add_labels: &[String],
    recursive: bool,
    updated_paths: &mut Vec<String>,
) {
    if add_labels
        .iter()
        .any(|label| !topic.labels.iter().any(|existing| existing == label))
    {
        updated_paths.push(path.to_selector_value());
    }

    if recursive {
        for child in &topic.children {
            let child_path = path.join(child.title.clone());
            collect_set_tree_metadata_updates(child, &child_path, add_labels, true, updated_paths);
        }
    }
}

fn collect_missing_path_additions(root: &Topic, target_path: &TopicPath) -> Vec<String> {
    let mut current = root;
    let mut current_path = TopicPath::root();
    let mut added_paths = Vec::new();
    let mut missing = false;

    for segment in target_path.segments() {
        if !missing {
            if let Some(child) = current
                .children
                .iter()
                .find(|child| child.title == segment.as_str())
            {
                current = child;
                current_path = current_path.join(segment.clone());
                continue;
            }
            missing = true;
        }

        current_path = current_path.join(segment.clone());
        added_paths.push(current_path.to_selector_value());
    }

    added_paths
}

fn collect_merge_tree_diff(
    topic: &Topic,
    path: &TopicPath,
    tree: &TopicTreeInputDto,
    match_by: &str,
    prune: bool,
    diff: &mut MergeTreeDiffAccumulator,
) {
    if merge_tree_updates_topic(topic, tree) {
        diff.updated_paths.push(path.to_selector_value());
    }

    for child_tree in &tree.children {
        if let Some(child_topic) = find_merge_tree_child(topic, path, child_tree, match_by) {
            collect_merge_tree_diff(
                child_topic,
                &path.join(child_topic.title.clone()),
                child_tree,
                match_by,
                prune,
                diff,
            );
        } else {
            diff.added_paths
                .extend(collect_added_paths(path, child_tree));
        }
    }

    if prune {
        for child_topic in &topic.children {
            if !tree.children.iter().any(|child_tree| {
                merge_tree_child_matches(
                    child_topic,
                    &path.join(child_topic.title.clone()),
                    child_tree,
                    match_by,
                )
            }) {
                diff.deleted_paths.extend(collect_deleted_paths(
                    child_topic,
                    &path.join(child_topic.title.clone()),
                ));
            }
        }
    }
}

fn find_merge_tree_child<'a>(
    topic: &'a Topic,
    parent_path: &TopicPath,
    child_tree: &TopicTreeInputDto,
    match_by: &str,
) -> Option<&'a Topic> {
    topic.children.iter().find(|child| {
        merge_tree_child_matches(
            child,
            &parent_path.join(child.title.clone()),
            child_tree,
            match_by,
        )
    })
}

fn merge_tree_child_matches(
    topic: &Topic,
    path: &TopicPath,
    tree: &TopicTreeInputDto,
    match_by: &str,
) -> bool {
    match match_by {
        "id" => tree.id.as_deref() == Some(topic.id.0.as_str()),
        "path" => tree.path.as_deref() == Some(path.to_selector_value().as_str()),
        _ => topic.title == tree.title,
    }
}

fn merge_tree_updates_topic(topic: &Topic, tree: &TopicTreeInputDto) -> bool {
    topic.title != tree.title
        || tree
            .note
            .as_ref()
            .is_some_and(|note| topic.note.as_deref() != Some(note.as_str()))
        || (!tree.labels.is_empty() && topic.labels != tree.labels)
        || (!tree.markers.is_empty() && topic.markers != tree.markers)
        || tree.image.is_some()
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
            .with_operation_field_context(
                index,
                op_name.to_owned(),
                format!("fields.{field}"),
            );
            return Err(render_error(invocation, json, error));
        }
    }

    Ok(())
}

fn patch_set_changes_topic(topic: &Topic, fields: &serde_json::Map<String, Value>) -> bool {
    fields.iter().any(|(field, value)| match field.as_str() {
        "title" => value.as_str() != Some(topic.title.as_str()),
        "note" => value.as_str() != topic.note.as_deref(),
        "hyperlink" => value.as_str() != topic.hyperlink.as_deref(),
        "labels" => {
            value
                .as_array()
                .map(|labels| {
                    labels
                        .iter()
                        .filter_map(|label| label.as_str().map(str::to_owned))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
                != topic.labels
        }
        "markers" => {
            value
                .as_array()
                .map(|markers| {
                    markers
                        .iter()
                        .filter_map(|marker| marker.as_str().map(str::to_owned))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
                != topic.markers
        }
        "image" if value.is_null() => topic.image.is_some(),
        "image" => true,
        _ => false,
    })
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
        .with_operation_field_context(index, op_name.to_owned(), "node");
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
