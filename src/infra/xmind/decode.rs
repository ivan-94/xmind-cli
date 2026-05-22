use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use crate::domain::sheet::{Sheet, SheetId};
use crate::domain::topic::{Topic, TopicId};
use crate::domain::workbook::Workbook;

pub fn read_workbook(path: &Path) -> Result<Workbook, XMindReadError> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut content = String::new();
    archive
        .by_name("content.json")
        .map_err(|_| XMindReadError::MissingContent)?
        .read_to_string(&mut content)?;

    let sheets: Vec<StorageSheet> = serde_json::from_str(&content)?;

    Ok(Workbook {
        sheets: sheets.into_iter().map(Into::into).collect(),
    })
}

#[derive(Debug, Error)]
pub enum XMindReadError {
    #[error("workbook package could not be opened: {0}")]
    Io(#[from] std::io::Error),

    #[error("workbook package is not a readable zip archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("workbook package does not contain content.json")]
    MissingContent,

    #[error("content.json could not be decoded: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageSheet {
    id: String,
    title: String,
    root_topic: StorageTopic,
}

impl From<StorageSheet> for Sheet {
    fn from(value: StorageSheet) -> Self {
        Self {
            id: SheetId(value.id),
            title: value.title,
            root: value.root_topic.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StorageTopic {
    id: String,
    title: String,
    notes: Option<StorageNotes>,
    #[serde(default)]
    children: StorageChildren,
}

#[derive(Debug, Deserialize)]
struct StorageNotes {
    plain: Option<StoragePlainNote>,
}

#[derive(Debug, Deserialize)]
struct StoragePlainNote {
    content: String,
}

impl From<StorageTopic> for Topic {
    fn from(value: StorageTopic) -> Self {
        Self {
            id: TopicId(value.id),
            title: value.title,
            note: value
                .notes
                .and_then(|notes| notes.plain.map(|plain| plain.content)),
            children: value
                .children
                .attached
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct StorageChildren {
    #[serde(default)]
    attached: Vec<StorageTopic>,
}
