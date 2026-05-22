use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::TopicTreeInputDto;

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
