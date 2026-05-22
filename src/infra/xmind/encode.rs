use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use thiserror::Error;
use zip::write::FileOptions;

pub fn append_child_topic(
    workbook_path: &Path,
    parent_topic_id: &str,
    title: &str,
    new_topic_id: &str,
) -> Result<(), XMindWriteError> {
    let file = File::open(workbook_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut entries = Vec::new();
    let mut content_json = None;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;

        if name == "content.json" {
            let mut content: Value = serde_json::from_slice(&bytes)?;
            if !append_topic_to_content(&mut content, parent_topic_id, title, new_topic_id) {
                return Err(XMindWriteError::ParentNotFound(parent_topic_id.to_owned()));
            }
            content_json = Some(serde_json::to_vec_pretty(&content)?);
        } else {
            entries.push((name, bytes));
        }
    }

    let Some(content_json) = content_json else {
        return Err(XMindWriteError::MissingContent);
    };

    let temp_path = temp_workbook_path(workbook_path);
    write_package(&temp_path, content_json, entries)?;
    fs::rename(&temp_path, workbook_path)?;

    Ok(())
}

pub fn rename_topic(
    workbook_path: &Path,
    topic_id: &str,
    new_title: &str,
) -> Result<(), XMindWriteError> {
    let file = File::open(workbook_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut entries = Vec::new();
    let mut content_json = None;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;

        if name == "content.json" {
            let mut content: Value = serde_json::from_slice(&bytes)?;
            if !rename_topic_in_content(&mut content, topic_id, new_title) {
                return Err(XMindWriteError::TopicNotFound(topic_id.to_owned()));
            }
            content_json = Some(serde_json::to_vec_pretty(&content)?);
        } else {
            entries.push((name, bytes));
        }
    }

    let Some(content_json) = content_json else {
        return Err(XMindWriteError::MissingContent);
    };

    let temp_path = temp_workbook_path(workbook_path);
    write_package(&temp_path, content_json, entries)?;
    fs::rename(&temp_path, workbook_path)?;

    Ok(())
}

fn append_topic_to_content(
    content: &mut Value,
    parent_topic_id: &str,
    title: &str,
    new_topic_id: &str,
) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| append_topic_to_topic(root, parent_topic_id, title, new_topic_id))
    })
}

fn rename_topic_in_content(content: &mut Value, topic_id: &str, new_title: &str) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| rename_topic_in_topic(root, topic_id, new_title))
    })
}

fn rename_topic_in_topic(topic: &mut Value, topic_id: &str, new_title: &str) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(topic_id) {
        if let Some(object) = topic.as_object_mut() {
            object.insert("title".to_owned(), Value::String(new_title.to_owned()));
            return true;
        }
        return false;
    }

    topic
        .get_mut("children")
        .and_then(|children| children.get_mut("attached"))
        .and_then(Value::as_array_mut)
        .is_some_and(|children| {
            children
                .iter_mut()
                .any(|child| rename_topic_in_topic(child, topic_id, new_title))
        })
}

fn append_topic_to_topic(
    topic: &mut Value,
    parent_topic_id: &str,
    title: &str,
    new_topic_id: &str,
) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(parent_topic_id) {
        let children = topic
            .as_object_mut()
            .expect("topic object was already accessed as an object")
            .entry("children")
            .or_insert_with(|| json!({ "attached": [] }));
        let attached = children
            .as_object_mut()
            .and_then(|children| children.get_mut("attached"))
            .and_then(Value::as_array_mut);

        if let Some(attached) = attached {
            attached.push(json!({
                "id": new_topic_id,
                "title": title,
            }));
            return true;
        }

        return false;
    }

    topic
        .get_mut("children")
        .and_then(|children| children.get_mut("attached"))
        .and_then(Value::as_array_mut)
        .is_some_and(|children| {
            children
                .iter_mut()
                .any(|child| append_topic_to_topic(child, parent_topic_id, title, new_topic_id))
        })
}

fn write_package(
    temp_path: &Path,
    content_json: Vec<u8>,
    entries: Vec<(String, Vec<u8>)>,
) -> Result<(), XMindWriteError> {
    let file = File::create(temp_path)?;
    let mut writer = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    writer.start_file("content.json", options)?;
    writer.write_all(&content_json)?;

    for (name, bytes) in entries {
        writer.start_file(name, options)?;
        writer.write_all(&bytes)?;
    }

    writer.finish()?;
    Ok(())
}

fn temp_workbook_path(workbook_path: &Path) -> PathBuf {
    let file_name = workbook_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workbook.xmind");
    workbook_path.with_file_name(format!("{file_name}.tmp"))
}

#[derive(Debug, Error)]
pub enum XMindWriteError {
    #[error("workbook package could not be read or written: {0}")]
    Io(#[from] std::io::Error),

    #[error("workbook package could not be decoded or written as zip: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("workbook package does not contain content.json")]
    MissingContent,

    #[error("content.json could not be decoded or encoded: {0}")]
    Json(#[from] serde_json::Error),

    #[error("parent topic was not found in content.json: {0}")]
    ParentNotFound(String),

    #[error("topic was not found in content.json: {0}")]
    TopicNotFound(String),
}
