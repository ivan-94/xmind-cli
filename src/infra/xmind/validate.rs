use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct ValidateResultDto {
    pub valid: bool,
    pub warnings: Vec<ValidationDiagnosticDto>,
    pub errors: Vec<ValidationDiagnosticDto>,
}

#[derive(Debug, Serialize)]
pub struct ValidationDiagnosticDto {
    pub code: String,
    pub message: String,
    pub path: String,
    pub suggested_fix: String,
}

impl ValidationDiagnosticDto {
    fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
        suggested_fix: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: path.into(),
            suggested_fix: suggested_fix.into(),
        }
    }

    fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
        suggested_fix: impl Into<String>,
    ) -> Self {
        Self::error(code, message, path, suggested_fix)
    }
}

#[derive(Debug, Error)]
pub enum ValidateReadError {
    #[error("workbook package could not be opened: {0}")]
    Io(#[from] std::io::Error),

    #[error("workbook package is not a readable zip archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("workbook package uses an unsupported XMind format variant")]
    UnsupportedFormat,

    #[error("content.json could not be decoded: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn validate_workbook_package(
    path: &Path,
    strict: bool,
) -> Result<ValidateResultDto, ValidateReadError> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut resources = BTreeSet::new();
    let mut has_content_json = false;
    let mut has_content_xml = false;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = entry.name();
        if name == "content.json" {
            has_content_json = true;
        }
        if name == "content.xml" {
            has_content_xml = true;
        }
        if name.starts_with("resources/") && !name.ends_with('/') {
            resources.insert(format!("xap:{name}"));
        }
    }

    if !has_content_json {
        if has_content_xml {
            return Err(ValidateReadError::UnsupportedFormat);
        }
        let errors = vec![ValidationDiagnosticDto::error(
            "missing_content",
            "Workbook package does not contain content.json.",
            "content.json",
            "Open and re-save the workbook with a supported XMind version, then retry.",
        )];
        return Ok(ValidateResultDto {
            valid: false,
            warnings: Vec::new(),
            errors,
        });
    }

    let mut content = String::new();
    archive
        .by_name("content.json")?
        .read_to_string(&mut content)?;
    let content: Value = serde_json::from_str(&content)?;

    let mut validator = StructuralValidator::new(resources);
    validator.validate_content(&content);
    let mut result = validator.into_result();
    if strict && !result.warnings.is_empty() {
        result.valid = false;
    }
    Ok(result)
}

struct StructuralValidator {
    resources: BTreeSet<String>,
    topic_ids: BTreeMap<String, String>,
    warnings: Vec<ValidationDiagnosticDto>,
    errors: Vec<ValidationDiagnosticDto>,
}

impl StructuralValidator {
    fn new(resources: BTreeSet<String>) -> Self {
        Self {
            resources,
            topic_ids: BTreeMap::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn validate_content(&mut self, content: &Value) {
        let Some(sheets) = content.as_array() else {
            self.errors.push(ValidationDiagnosticDto::error(
                "missing_sheet",
                "content.json must be an array of sheets.",
                "content.json",
                "Open and re-save the workbook, or regenerate content.json as a sheet array.",
            ));
            return;
        };

        if sheets.is_empty() {
            self.errors.push(ValidationDiagnosticDto::error(
                "missing_sheet",
                "Workbook must contain at least one sheet.",
                "content.json",
                "Create at least one sheet before retrying validation.",
            ));
        }

        for (index, sheet) in sheets.iter().enumerate() {
            self.validate_sheet(sheet, &format!("content.json[{index}]"));
        }
    }

    fn validate_sheet(&mut self, sheet: &Value, path: &str) {
        let Some(sheet) = sheet.as_object() else {
            self.errors.push(ValidationDiagnosticDto::error(
                "missing_sheet",
                "Sheet entry must be an object.",
                path,
                "Open and re-save the workbook, or remove the invalid sheet entry.",
            ));
            return;
        };

        self.validate_required_string(sheet.get("id"), &format!("{path}.id"), "sheet id");
        self.validate_required_string(sheet.get("title"), &format!("{path}.title"), "sheet title");

        let root_path = format!("{path}.rootTopic");
        match sheet.get("rootTopic").and_then(Value::as_object) {
            Some(root) => self.validate_topic(root, &root_path),
            None => self.errors.push(ValidationDiagnosticDto::error(
                "missing_required_field",
                "Sheet is missing required rootTopic.",
                root_path,
                "Open and re-save the workbook, or restore the sheet root topic.",
            )),
        }

        if let Some(relationships) = sheet.get("relationships") {
            self.validate_relationships(relationships, &format!("{path}.relationships"));
        }
    }

    fn validate_topic(&mut self, topic: &serde_json::Map<String, Value>, path: &str) {
        if let Some(topic_id) =
            self.validate_required_string(topic.get("id"), &format!("{path}.id"), "topic id")
        {
            if let Some(previous_path) = self.topic_ids.insert(topic_id.clone(), path.to_owned()) {
                self.errors.push(ValidationDiagnosticDto::error(
                    "duplicate_topic_id",
                    format!("Topic id `{topic_id}` appears more than once."),
                    format!("{path}.id ({topic_id})"),
                    format!(
                        "Assign a unique id to this topic; first occurrence is at {previous_path}.",
                    ),
                ));
            }
        }

        self.validate_required_string(topic.get("title"), &format!("{path}.title"), "topic title");
        self.validate_topic_image(topic, path);

        if let Some(children) = topic.get("children").and_then(Value::as_object) {
            if let Some(attached) = children.get("attached") {
                match attached.as_array() {
                    Some(attached) => {
                        for (index, child) in attached.iter().enumerate() {
                            let child_path = format!("{path}.children.attached[{index}]");
                            match child.as_object() {
                                Some(child) => self.validate_topic(child, &child_path),
                                None => self.errors.push(ValidationDiagnosticDto::error(
                                    "invalid_topic_order",
                                    "Attached child topic must be an object.",
                                    child_path,
                                    "Open and re-save the workbook, or remove the invalid child entry.",
                                )),
                            }
                        }
                    }
                    None => self.errors.push(ValidationDiagnosticDto::error(
                        "invalid_topic_order",
                        "children.attached must be an array when present.",
                        format!("{path}.children.attached"),
                        "Open and re-save the workbook, or restore attached children as an array.",
                    )),
                }
            }
        }
    }

    fn validate_topic_image(&mut self, topic: &serde_json::Map<String, Value>, path: &str) {
        let Some(image) = topic.get("image").and_then(Value::as_object) else {
            return;
        };
        let Some(src) = image.get("src").and_then(Value::as_str) else {
            return;
        };
        if src.starts_with("xap:resources/") && !self.resources.contains(src) {
            self.warnings.push(ValidationDiagnosticDto::warning(
                "missing_resource",
                format!("Topic image references missing resource `{src}`."),
                format!("{path}.image.src"),
                "Restore the missing package resource or remove the image reference.",
            ));
        }
    }

    fn validate_relationships(&mut self, relationships: &Value, path: &str) {
        let Some(relationships) = relationships.as_array() else {
            self.errors.push(ValidationDiagnosticDto::error(
                "invalid_relationships",
                "relationships must be an array when present.",
                path,
                "Open and re-save the workbook, or remove the invalid relationships entry.",
            ));
            return;
        };

        for (index, relationship) in relationships.iter().enumerate() {
            let relationship_path = format!("{path}[{index}]");
            let Some(relationship) = relationship.as_object() else {
                self.errors.push(ValidationDiagnosticDto::error(
                    "invalid_relationships",
                    "Relationship entry must be an object.",
                    relationship_path,
                    "Open and re-save the workbook, or remove the invalid relationship entry.",
                ));
                continue;
            };

            for endpoint in ["end1Id", "end2Id"] {
                let endpoint_path = format!("{relationship_path}.{endpoint}");
                match relationship.get(endpoint).and_then(Value::as_str) {
                    Some(topic_id) if self.topic_ids.contains_key(topic_id) => {}
                    Some(topic_id) => self.errors.push(ValidationDiagnosticDto::error(
                        "broken_relationship_reference",
                        format!("Relationship endpoint `{topic_id}` does not match any topic id."),
                        endpoint_path,
                        "Update the relationship endpoint to an existing topic id or remove the relationship.",
                    )),
                    None => self.errors.push(ValidationDiagnosticDto::error(
                        "missing_required_field",
                        format!("Relationship is missing required {endpoint}."),
                        endpoint_path,
                        "Open and re-save the workbook, or restore the relationship endpoint.",
                    )),
                }
            }
        }
    }

    fn validate_required_string(
        &mut self,
        value: Option<&Value>,
        path: &str,
        label: &str,
    ) -> Option<String> {
        match value.and_then(Value::as_str) {
            Some(value) if !value.trim().is_empty() => Some(value.to_owned()),
            _ => {
                self.errors.push(ValidationDiagnosticDto::error(
                    "missing_required_field",
                    format!("Missing required {label}."),
                    path,
                    format!("Open and re-save the workbook, or restore the {label}."),
                ));
                None
            }
        }
    }

    fn into_result(self) -> ValidateResultDto {
        ValidateResultDto {
            valid: self.errors.is_empty(),
            warnings: self.warnings,
            errors: self.errors,
        }
    }
}
