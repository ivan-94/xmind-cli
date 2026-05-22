#![allow(dead_code)]

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;
use zip::write::FileOptions;

use crate::domain::sheet::Sheet;
use crate::domain::topic::{Topic, TopicImageRef};
use crate::domain::workbook::Workbook;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopicClearField {
    Note,
    Labels,
    Markers,
    Hyperlink,
}

pub fn encode_workbook_content(workbook: &Workbook) -> Result<Vec<u8>, XMindWriteError> {
    let sheets = workbook
        .sheets
        .iter()
        .map(StorageSheetDto::from_sheet)
        .collect::<Vec<_>>();
    let mut content = serde_json::to_value(sheets)?;
    merge_preserved_json_fields(&mut content, workbook);

    Ok(serde_json::to_vec_pretty(&content)?)
}

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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn set_topic_note(
    workbook_path: &Path,
    topic_id: &str,
    note: &str,
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
            if !set_topic_note_in_content(&mut content, topic_id, note) {
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn set_topic_labels(
    workbook_path: &Path,
    topic_id: &str,
    labels: &[String],
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
            if !set_topic_labels_in_content(&mut content, topic_id, labels) {
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn set_topic_markers(
    workbook_path: &Path,
    topic_id: &str,
    markers: &[String],
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
            if !set_topic_markers_in_content(&mut content, topic_id, markers) {
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn set_topic_hyperlink(
    workbook_path: &Path,
    topic_id: &str,
    hyperlink: &str,
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
            if !set_topic_hyperlink_in_content(&mut content, topic_id, hyperlink) {
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn clear_topic_fields(
    workbook_path: &Path,
    topic_id: &str,
    fields: &[TopicClearField],
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
            if !clear_topic_fields_in_content(&mut content, topic_id, fields) {
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn delete_topic(workbook_path: &Path, topic_id: &str) -> Result<(), XMindWriteError> {
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
            if !delete_topic_in_content(&mut content, topic_id) {
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn delete_topic_children(workbook_path: &Path, topic_id: &str) -> Result<(), XMindWriteError> {
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
            if !delete_topic_children_in_content(&mut content, topic_id) {
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn delete_topic_promote_children(
    workbook_path: &Path,
    topic_id: &str,
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
            if !delete_topic_promote_children_in_content(&mut content, topic_id) {
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn move_topic(
    workbook_path: &Path,
    topic_id: &str,
    destination_topic_id: &str,
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
            let Some(topic) = remove_topic_from_content(&mut content, topic_id) else {
                return Err(XMindWriteError::TopicNotFound(topic_id.to_owned()));
            };
            if !append_existing_topic_to_content(&mut content, destination_topic_id, topic) {
                return Err(XMindWriteError::TopicNotFound(
                    destination_topic_id.to_owned(),
                ));
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

    Ok(())
}

pub fn copy_topic(
    workbook_path: &Path,
    source_topic_id: &str,
    destination_topic_id: &str,
    new_root_topic_id: &str,
    new_root_title: &str,
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
            let Some(source) = find_topic_in_content(&content, source_topic_id) else {
                return Err(XMindWriteError::TopicNotFound(source_topic_id.to_owned()));
            };
            let mut copied = source.clone();
            rewrite_copied_topic_ids(&mut copied, new_root_topic_id, Some(new_root_title));

            if !append_existing_topic_to_content(&mut content, destination_topic_id, copied) {
                return Err(XMindWriteError::TopicNotFound(
                    destination_topic_id.to_owned(),
                ));
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
    replace_with_validated_candidate(workbook_path, &temp_path, validate_candidate_package)?;

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

fn set_topic_note_in_content(content: &mut Value, topic_id: &str, note: &str) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| set_topic_note_in_topic(root, topic_id, note))
    })
}

fn set_topic_labels_in_content(content: &mut Value, topic_id: &str, labels: &[String]) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| set_topic_labels_in_topic(root, topic_id, labels))
    })
}

fn set_topic_markers_in_content(content: &mut Value, topic_id: &str, markers: &[String]) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| set_topic_markers_in_topic(root, topic_id, markers))
    })
}

fn set_topic_hyperlink_in_content(content: &mut Value, topic_id: &str, hyperlink: &str) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| set_topic_hyperlink_in_topic(root, topic_id, hyperlink))
    })
}

fn clear_topic_fields_in_content(
    content: &mut Value,
    topic_id: &str,
    fields: &[TopicClearField],
) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| clear_topic_fields_in_topic(root, topic_id, fields))
    })
}

fn delete_topic_in_content(content: &mut Value, topic_id: &str) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| delete_topic_below(root, topic_id))
    })
}

fn delete_topic_children_in_content(content: &mut Value, topic_id: &str) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| delete_topic_children_in_topic(root, topic_id))
    })
}

fn delete_topic_promote_children_in_content(content: &mut Value, topic_id: &str) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    sheets.iter_mut().any(|sheet| {
        sheet
            .get_mut("rootTopic")
            .is_some_and(|root| delete_topic_promote_children_below(root, topic_id))
    })
}

fn remove_topic_from_content(content: &mut Value, topic_id: &str) -> Option<Value> {
    let sheets = content.as_array_mut()?;

    sheets.iter_mut().find_map(|sheet| {
        sheet
            .get_mut("rootTopic")
            .and_then(|root| remove_topic_below(root, topic_id))
    })
}

fn append_existing_topic_to_content(
    content: &mut Value,
    destination_topic_id: &str,
    topic: Value,
) -> bool {
    let Some(sheets) = content.as_array_mut() else {
        return false;
    };

    let mut topic = Some(topic);
    sheets.iter_mut().any(|sheet| {
        sheet.get_mut("rootTopic").is_some_and(|root| {
            append_existing_topic_to_topic(root, destination_topic_id, &mut topic)
        })
    })
}

fn find_topic_in_content<'a>(content: &'a Value, topic_id: &str) -> Option<&'a Value> {
    let sheets = content.as_array()?;

    sheets.iter().find_map(|sheet| {
        sheet
            .get("rootTopic")
            .and_then(|root| find_topic_in_topic(root, topic_id))
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

fn set_topic_labels_in_topic(topic: &mut Value, topic_id: &str, labels: &[String]) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(topic_id) {
        if let Some(object) = topic.as_object_mut() {
            object.insert("labels".to_owned(), json!(labels));
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
                .any(|child| set_topic_labels_in_topic(child, topic_id, labels))
        })
}

fn set_topic_markers_in_topic(topic: &mut Value, topic_id: &str, markers: &[String]) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(topic_id) {
        if let Some(object) = topic.as_object_mut() {
            object.insert(
                "markers".to_owned(),
                Value::Array(
                    markers
                        .iter()
                        .map(|marker| json!({ "markerId": marker }))
                        .collect(),
                ),
            );
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
                .any(|child| set_topic_markers_in_topic(child, topic_id, markers))
        })
}

fn set_topic_hyperlink_in_topic(topic: &mut Value, topic_id: &str, hyperlink: &str) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(topic_id) {
        if let Some(object) = topic.as_object_mut() {
            object.insert("href".to_owned(), Value::String(hyperlink.to_owned()));
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
                .any(|child| set_topic_hyperlink_in_topic(child, topic_id, hyperlink))
        })
}

fn clear_topic_fields_in_topic(
    topic: &mut Value,
    topic_id: &str,
    fields: &[TopicClearField],
) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(topic_id) {
        if let Some(object) = topic.as_object_mut() {
            for field in fields {
                match field {
                    TopicClearField::Note => {
                        object.remove("notes");
                    }
                    TopicClearField::Labels => {
                        object.remove("labels");
                    }
                    TopicClearField::Markers => {
                        object.remove("markers");
                    }
                    TopicClearField::Hyperlink => {
                        object.remove("href");
                    }
                }
            }
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
                .any(|child| clear_topic_fields_in_topic(child, topic_id, fields))
        })
}

fn delete_topic_below(topic: &mut Value, topic_id: &str) -> bool {
    let Some(children) = topic
        .get_mut("children")
        .and_then(|children| children.get_mut("attached"))
        .and_then(Value::as_array_mut)
    else {
        return false;
    };

    if let Some(index) = children
        .iter()
        .position(|child| child.get("id").and_then(Value::as_str) == Some(topic_id))
    {
        children.remove(index);
        return true;
    }

    children
        .iter_mut()
        .any(|child| delete_topic_below(child, topic_id))
}

fn delete_topic_children_in_topic(topic: &mut Value, topic_id: &str) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(topic_id) {
        if let Some(children) = topic
            .get_mut("children")
            .and_then(|children| children.get_mut("attached"))
            .and_then(Value::as_array_mut)
        {
            children.clear();
        }
        return true;
    }

    topic
        .get_mut("children")
        .and_then(|children| children.get_mut("attached"))
        .and_then(Value::as_array_mut)
        .is_some_and(|children| {
            children
                .iter_mut()
                .any(|child| delete_topic_children_in_topic(child, topic_id))
        })
}

fn delete_topic_promote_children_below(topic: &mut Value, topic_id: &str) -> bool {
    let Some(children) = topic
        .get_mut("children")
        .and_then(|children| children.get_mut("attached"))
        .and_then(Value::as_array_mut)
    else {
        return false;
    };

    if let Some(index) = children
        .iter()
        .position(|child| child.get("id").and_then(Value::as_str) == Some(topic_id))
    {
        let mut removed = children.remove(index);
        let promoted = removed
            .get_mut("children")
            .and_then(|children| children.get_mut("attached"))
            .and_then(Value::as_array_mut)
            .map(std::mem::take)
            .unwrap_or_default();

        for (offset, child) in promoted.into_iter().enumerate() {
            children.insert(index + offset, child);
        }
        return true;
    }

    children
        .iter_mut()
        .any(|child| delete_topic_promote_children_below(child, topic_id))
}

fn remove_topic_below(topic: &mut Value, topic_id: &str) -> Option<Value> {
    let children = topic
        .get_mut("children")
        .and_then(|children| children.get_mut("attached"))
        .and_then(Value::as_array_mut)?;

    if let Some(index) = children
        .iter()
        .position(|child| child.get("id").and_then(Value::as_str) == Some(topic_id))
    {
        return Some(children.remove(index));
    }

    children
        .iter_mut()
        .find_map(|child| remove_topic_below(child, topic_id))
}

fn append_existing_topic_to_topic(
    topic: &mut Value,
    destination_topic_id: &str,
    moved_topic: &mut Option<Value>,
) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(destination_topic_id) {
        if let Some(moved_topic) = moved_topic.take() {
            let children = topic
                .as_object_mut()
                .expect("topic matched by id is an object")
                .entry("children")
                .or_insert_with(|| json!({ "attached": [] }));
            let attached = children
                .as_object_mut()
                .and_then(|children| children.get_mut("attached"))
                .and_then(Value::as_array_mut);

            if let Some(attached) = attached {
                attached.push(moved_topic);
                return true;
            }
        }
        return false;
    }

    topic
        .get_mut("children")
        .and_then(|children| children.get_mut("attached"))
        .and_then(Value::as_array_mut)
        .is_some_and(|children| {
            children.iter_mut().any(|child| {
                append_existing_topic_to_topic(child, destination_topic_id, moved_topic)
            })
        })
}

fn find_topic_in_topic<'a>(topic: &'a Value, topic_id: &str) -> Option<&'a Value> {
    if topic.get("id").and_then(Value::as_str) == Some(topic_id) {
        return Some(topic);
    }

    topic
        .get("children")
        .and_then(|children| children.get("attached"))
        .and_then(Value::as_array)
        .and_then(|children| {
            children
                .iter()
                .find_map(|child| find_topic_in_topic(child, topic_id))
        })
}

fn rewrite_copied_topic_ids(topic: &mut Value, new_id: &str, new_title: Option<&str>) {
    if let Some(object) = topic.as_object_mut() {
        let original_id = object
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or(new_id)
            .to_owned();
        object.insert("id".to_owned(), Value::String(new_id.to_owned()));
        if let Some(new_title) = new_title {
            object.insert("title".to_owned(), Value::String(new_title.to_owned()));
        }

        if let Some(children) = object
            .get_mut("children")
            .and_then(|children| children.get_mut("attached"))
            .and_then(Value::as_array_mut)
        {
            for child in children {
                let child_new_id = child
                    .get("id")
                    .and_then(Value::as_str)
                    .map(|id| format!("{id}-copy"))
                    .unwrap_or_else(|| format!("{original_id}-copy-child"));
                rewrite_copied_topic_ids(child, &child_new_id, None);
            }
        }
    }
}

fn set_topic_note_in_topic(topic: &mut Value, topic_id: &str, note: &str) -> bool {
    if topic.get("id").and_then(Value::as_str) == Some(topic_id) {
        if let Some(object) = topic.as_object_mut() {
            object.insert(
                "notes".to_owned(),
                json!({
                    "plain": {
                        "content": note,
                    },
                }),
            );
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
                .any(|child| set_topic_note_in_topic(child, topic_id, note))
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

fn validate_candidate_package(candidate_path: &Path) -> Result<(), XMindWriteError> {
    crate::infra::xmind::decode::read_workbook(candidate_path)
        .map(|_| ())
        .map_err(|error| XMindWriteError::CandidateValidationFailed(error.to_string()))
}

fn replace_with_validated_candidate<F>(
    workbook_path: &Path,
    candidate_path: &Path,
    validate_candidate: F,
) -> Result<(), XMindWriteError>
where
    F: FnOnce(&Path) -> Result<(), XMindWriteError>,
{
    validate_candidate(candidate_path)?;
    fs::rename(candidate_path, workbook_path)?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageSheetDto {
    id: String,
    title: String,
    root_topic: StorageTopicDto,
}

impl StorageSheetDto {
    fn from_sheet(sheet: &Sheet) -> Self {
        Self {
            id: sheet.id.0.clone(),
            title: sheet.title.clone(),
            root_topic: StorageTopicDto::from_topic(&sheet.root),
        }
    }
}

#[derive(Serialize)]
struct StorageTopicDto {
    id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<StorageNotesDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<StorageImageDto>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    labels: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    markers: Vec<StorageMarkerDto>,
    children: StorageChildrenDto,
}

impl StorageTopicDto {
    fn from_topic(topic: &Topic) -> Self {
        Self {
            id: topic.id.0.clone(),
            title: topic.title.clone(),
            notes: topic.note.as_ref().map(|note| StorageNotesDto {
                plain: StoragePlainNoteDto {
                    content: note.clone(),
                },
            }),
            href: topic.hyperlink.clone(),
            image: topic.image.as_ref().map(StorageImageDto::from_topic_image),
            labels: topic.labels.clone(),
            markers: topic
                .markers
                .iter()
                .map(|marker| StorageMarkerDto {
                    marker_id: marker.clone(),
                })
                .collect(),
            children: StorageChildrenDto {
                attached: topic
                    .children
                    .iter()
                    .map(StorageTopicDto::from_topic)
                    .collect(),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageMarkerDto {
    marker_id: String,
}

#[derive(Serialize)]
struct StorageImageDto {
    src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    alt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

impl StorageImageDto {
    fn from_topic_image(image: &TopicImageRef) -> Self {
        Self {
            src: image.asset_id.as_str().to_owned(),
            alt: image.alt.clone(),
            title: image.title.clone(),
        }
    }
}

#[derive(Serialize)]
struct StorageNotesDto {
    plain: StoragePlainNoteDto,
}

#[derive(Serialize)]
struct StoragePlainNoteDto {
    content: String,
}

#[derive(Serialize)]
struct StorageChildrenDto {
    attached: Vec<StorageTopicDto>,
}

fn merge_preserved_json_fields(content: &mut Value, workbook: &Workbook) {
    for (path, value) in workbook.preservation.raw_json_fields() {
        merge_preserved_json_field(content, path, value.clone());
    }
}

fn merge_preserved_json_field(content: &mut Value, path: &str, value: Value) {
    let mut parts = path.split('.').collect::<Vec<_>>();
    let Some(field_name) = parts.pop() else {
        return;
    };

    let Some(parent) = navigate_preserved_parent(content, &parts) else {
        return;
    };

    if let Some(object) = parent.as_object_mut() {
        object.insert(field_name.to_owned(), value);
    }
}

fn navigate_preserved_parent<'a>(
    mut current: &'a mut Value,
    parts: &[&str],
) -> Option<&'a mut Value> {
    for part in parts {
        current = navigate_preserved_part(current, part)?;
    }
    Some(current)
}

fn navigate_preserved_part<'a>(current: &'a mut Value, part: &str) -> Option<&'a mut Value> {
    if let Some((name, index)) = parse_indexed_part(part) {
        if name == "sheets" {
            return current.as_array_mut()?.get_mut(index);
        }
        return current
            .as_object_mut()?
            .get_mut(name)?
            .as_array_mut()?
            .get_mut(index);
    }

    current.as_object_mut()?.get_mut(part)
}

fn parse_indexed_part(part: &str) -> Option<(&str, usize)> {
    let (name, index) = part.strip_suffix(']')?.split_once('[')?;
    Some((name, index.parse().ok()?))
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

    #[error("candidate workbook failed validation: {0}")]
    CandidateValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use crate::domain::sheet::{Sheet, SheetId};
    use crate::domain::topic::{AssetId, Topic, TopicId, TopicImageRef};
    use crate::domain::workbook::{PreservationBag, ResourceIndex, Workbook};

    use super::{
        encode_workbook_content, replace_with_validated_candidate, validate_candidate_package,
        Value, XMindWriteError,
    };

    #[test]
    fn encodes_domain_workbook_to_supported_storage_dtos() {
        let workbook = Workbook {
            sheets: vec![Sheet {
                id: SheetId("sheet-roadmap".to_owned()),
                title: "Roadmap".to_owned(),
                root: Topic {
                    id: TopicId("topic-root".to_owned()),
                    title: "Roadmap".to_owned(),
                    note: Some("Root note".to_owned()),
                    labels: vec!["plan".to_owned()],
                    markers: vec!["priority-1".to_owned()],
                    hyperlink: Some("https://example.com".to_owned()),
                    image: Some(TopicImageRef::new(
                        AssetId::new("xap:resources/root.png"),
                        Some("Root image".to_owned()),
                        Some("Root".to_owned()),
                    )),
                    children: vec![Topic {
                        id: TopicId("topic-payment".to_owned()),
                        title: "Payment".to_owned(),
                        note: None,
                        labels: Vec::new(),
                        markers: Vec::new(),
                        hyperlink: None,
                        image: None,
                        children: Vec::new(),
                    }],
                },
            }],
            resources: ResourceIndex::default(),
            preservation: PreservationBag::default(),
        };

        let content = encode_workbook_content(&workbook).expect("workbook encodes");
        let value: Value = serde_json::from_slice(&content).expect("encoded content is JSON");

        assert_eq!(value[0]["id"], "sheet-roadmap");
        assert_eq!(value[0]["title"], "Roadmap");
        assert_eq!(value[0]["rootTopic"]["id"], "topic-root");
        assert_eq!(value[0]["rootTopic"]["title"], "Roadmap");
        assert_eq!(
            value[0]["rootTopic"]["notes"]["plain"]["content"],
            "Root note"
        );
        assert_eq!(value[0]["rootTopic"]["labels"][0], "plan");
        assert_eq!(
            value[0]["rootTopic"]["markers"][0]["markerId"],
            "priority-1"
        );
        assert_eq!(value[0]["rootTopic"]["href"], "https://example.com");
        assert_eq!(
            value[0]["rootTopic"]["image"]["src"],
            "xap:resources/root.png"
        );
        assert_eq!(value[0]["rootTopic"]["image"]["alt"], "Root image");
        assert_eq!(value[0]["rootTopic"]["image"]["title"], "Root");
        assert_eq!(
            value[0]["rootTopic"]["children"]["attached"][0]["id"],
            "topic-payment"
        );
    }

    #[test]
    fn merges_preserved_unknown_json_fields_into_encoded_content() {
        let mut preservation = PreservationBag::default();
        preservation.preserve_json_field(
            "sheets[0].vendorSheet",
            serde_json::json!({ "sheet": true }),
        );
        preservation.preserve_json_field(
            "sheets[0].rootTopic.vendorTopic",
            serde_json::json!({ "topic": true }),
        );
        preservation.preserve_json_field(
            "sheets[0].rootTopic.children.attached[0].vendorChild",
            serde_json::json!({ "child": true }),
        );

        let workbook = Workbook {
            sheets: vec![Sheet {
                id: SheetId("sheet-roadmap".to_owned()),
                title: "Roadmap".to_owned(),
                root: Topic {
                    id: TopicId("topic-root".to_owned()),
                    title: "Roadmap".to_owned(),
                    note: None,
                    labels: Vec::new(),
                    markers: Vec::new(),
                    hyperlink: None,
                    image: None,
                    children: vec![Topic {
                        id: TopicId("topic-payment".to_owned()),
                        title: "Payment".to_owned(),
                        note: None,
                        labels: Vec::new(),
                        markers: Vec::new(),
                        hyperlink: None,
                        image: None,
                        children: Vec::new(),
                    }],
                },
            }],
            resources: ResourceIndex::default(),
            preservation,
        };

        let content = encode_workbook_content(&workbook).expect("workbook encodes");
        let value: Value = serde_json::from_slice(&content).expect("encoded content is JSON");

        assert_eq!(value[0]["vendorSheet"]["sheet"], true);
        assert_eq!(value[0]["rootTopic"]["vendorTopic"]["topic"], true);
        assert_eq!(
            value[0]["rootTopic"]["children"]["attached"][0]["vendorChild"]["child"],
            true
        );
    }

    #[test]
    fn candidate_validation_rejects_malformed_content_package() {
        use std::fs::File;
        use std::io::Write;

        use zip::write::FileOptions;

        let temp_dir = tempfile::tempdir().expect("temp dir is created");
        let candidate = temp_dir.path().join("candidate.xmind");
        let file = File::create(&candidate).expect("candidate is created");
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("content.json", options)
            .expect("content entry starts");
        zip.write_all(br#"not-json"#)
            .expect("malformed content is written");
        zip.finish().expect("zip finishes");

        let error = validate_candidate_package(&candidate).expect_err("candidate is invalid");

        assert!(matches!(
            error,
            XMindWriteError::CandidateValidationFailed(_)
        ));
    }

    #[test]
    fn validated_replace_leaves_original_file_untouched_when_validation_fails() {
        let temp_dir = tempfile::tempdir().expect("temp dir is created");
        let workbook = temp_dir.path().join("roadmap.xmind");
        let candidate = temp_dir.path().join("roadmap.xmind.tmp");
        std::fs::write(&workbook, b"original workbook").expect("original is written");
        std::fs::write(&candidate, b"candidate workbook").expect("candidate is written");

        let error = replace_with_validated_candidate(&workbook, &candidate, |_| {
            Err(XMindWriteError::CandidateValidationFailed(
                "forced validation failure".to_owned(),
            ))
        })
        .expect_err("candidate validation failure prevents replace");

        assert!(matches!(
            error,
            XMindWriteError::CandidateValidationFailed(_)
        ));
        assert_eq!(
            std::fs::read(&workbook).expect("original is readable"),
            b"original workbook"
        );
    }
}
