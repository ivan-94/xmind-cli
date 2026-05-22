use crate::cli::{CandidateDto, CliErrorBody, CommandEnvelope, ErrorCode};
use crate::domain::diff::{Diff, DiffEvent, FieldChange};
use crate::domain::path::TopicPath;
use crate::domain::selector::Selector;
use crate::infra::xmind::encode::TopicClearField;
use crate::render::diff::render_human_outline;

use super::{
    create_mutation_backup, read_workbook_or_render_error, render_backup_error, render_error,
    render_workbook_write_error, resolve_topic, select_sheet_or_render_error, DiffEventDto,
    Invocation, ResolveOne, SetTitleDryRunResultDto, SummaryDto, UpdatedTopicDto,
};

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
            "image" => TopicClearField::Image,
            _ => {
                return Err(CliErrorBody::new(
                    ErrorCode::InvalidUsage,
                    format!("Unknown --clear field: {field}"),
                    true,
                    "Use --clear note, --clear labels, --clear markers, --clear hyperlink, or --clear image.",
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
        TopicClearField::Image => "image",
    }
}

pub(super) fn render_set_title(invocation: Invocation, json: bool, node: &str, title: &str) -> i32 {
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

pub(super) fn render_set_note(invocation: Invocation, json: bool, node: &str, note: &str) -> i32 {
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

pub(super) fn render_set_labels(
    invocation: Invocation,
    json: bool,
    node: &str,
    labels: Vec<String>,
) -> i32 {
    render_set_label_mutation(invocation, json, node, LabelMutation::Replace(labels))
}

pub(super) fn render_set_add_label(
    invocation: Invocation,
    json: bool,
    node: &str,
    label: String,
) -> i32 {
    render_set_label_mutation(invocation, json, node, LabelMutation::Add(label))
}

pub(super) fn render_set_remove_label(
    invocation: Invocation,
    json: bool,
    node: &str,
    label: String,
) -> i32 {
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

pub(super) fn render_set_markers(
    invocation: Invocation,
    json: bool,
    node: &str,
    markers: Vec<String>,
) -> i32 {
    render_set_marker_mutation(invocation, json, node, MarkerMutation::Replace(markers))
}

pub(super) fn render_set_add_marker(
    invocation: Invocation,
    json: bool,
    node: &str,
    marker: String,
) -> i32 {
    render_set_marker_mutation(invocation, json, node, MarkerMutation::Add(marker))
}

pub(super) fn render_set_remove_marker(
    invocation: Invocation,
    json: bool,
    node: &str,
    marker: String,
) -> i32 {
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

pub(super) fn render_set_hyperlink(
    invocation: Invocation,
    json: bool,
    node: &str,
    hyperlink: &str,
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

pub(super) fn render_set_clear(
    invocation: Invocation,
    json: bool,
    node: &str,
    fields: Vec<String>,
) -> i32 {
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
        TopicClearField::Image => resolved.topic.image.is_some(),
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

pub(super) fn render_set_append_note(
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
