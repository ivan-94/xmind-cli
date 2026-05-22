use std::fs;
use std::path::Path;

use crate::cli::{CandidateDto, CliErrorBody, CommandEnvelope, ErrorCode};
use crate::domain::diff::{Diff, DiffEvent, FieldChange};
use crate::domain::selector::Selector;
use crate::infra::xmind::assets::detect_supported_image_media_type;
use crate::render::diff::render_human_outline;

use super::{
    create_mutation_backup, read_workbook_or_render_error, render_backup_error, render_error,
    render_workbook_write_error, resolve_topic, select_sheet_or_render_error, DiffEventDto,
    Invocation, ResolveOne, SetTitleDryRunResultDto, SummaryDto, UpdatedTopicDto,
};

pub(super) fn render_set_image(
    invocation: Invocation,
    json: bool,
    node: &str,
    image: &Path,
    alt: Option<String>,
    title: Option<String>,
) -> i32 {
    let Some(file_name) = image.file_name().and_then(|name| name.to_str()) else {
        let error = CliErrorBody::new(
            ErrorCode::InvalidUsage,
            "Image path must include a file name.",
            true,
            "Pass a local image file path such as ./diagram.png.",
        )
        .with_path(image.display().to_string());
        return render_error(invocation, json, error);
    };
    let asset_entry_name = format!("resources/{file_name}");
    let asset_id = format!("xap:{asset_entry_name}");
    let image_bytes = match fs::read(image) {
        Ok(bytes) => bytes,
        Err(error) => {
            let error = CliErrorBody::new(
                ErrorCode::FileNotFound,
                format!("Image file could not be read: {error}"),
                true,
                "Check the image path and retry.",
            )
            .with_path(image.display().to_string());
            return render_error(invocation, json, error);
        }
    };
    if detect_supported_image_media_type(&image_bytes).is_none() {
        let error = CliErrorBody::new(
            ErrorCode::UnsupportedAssetType,
            "Only PNG, JPEG, GIF, and SVG topic images are supported for attachment.",
            false,
            "Convert the image to PNG or JPEG and retry.",
        )
        .with_path(image.display().to_string());
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

    let path = resolved.path.to_selector_value();
    let human_diff = Diff::from_events(vec![DiffEvent::Updated {
        path: resolved.path.clone(),
        fields: vec![FieldChange::new("image")],
    }]);
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
            changed_fields: vec!["image"],
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
        if let Err(error) = crate::infra::xmind::encode::set_topic_image(
            &invocation.workbook,
            &resolved.topic.id.0,
            &asset_id,
            alt.as_deref(),
            title.as_deref(),
            &asset_entry_name,
            image_bytes,
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
