use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::domain::sheet::{Sheet, SheetId};
use crate::domain::topic::{AssetId, Topic, TopicId, TopicImageRef};
use crate::domain::workbook::{PreservationBag, ResourceIndex, Workbook};

pub fn read_workbook(path: &Path) -> Result<Workbook, XMindReadError> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    if archive.by_name("content.json").is_err() && archive.by_name("content.xml").is_ok() {
        return Err(XMindReadError::UnsupportedFormat);
    }

    let mut preservation = PreservationBag::default();
    let mut resources = ResourceIndex::default();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        if name != "content.json" {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            preservation.preserve_package_entry_bytes(name.clone(), bytes);
        }
        if is_resource_entry(&name) {
            resources.insert_asset_id(AssetId::new(format!("xap:{name}")));
        }
    }

    let mut content = String::new();
    archive
        .by_name("content.json")
        .map_err(|_| XMindReadError::MissingContent)?
        .read_to_string(&mut content)?;

    let sheets: Vec<StorageSheet> = serde_json::from_str(&content)?;
    for (sheet_index, sheet) in sheets.iter().enumerate() {
        sheet.preserve_unknown_json_fields(&mut preservation, &format!("sheets[{sheet_index}]"));
    }

    Ok(Workbook {
        sheets: sheets.into_iter().map(Into::into).collect(),
        resources,
        preservation,
    })
}

fn is_resource_entry(name: &str) -> bool {
    name.starts_with("resources/") && !name.ends_with('/')
}

#[derive(Debug, Error)]
pub enum XMindReadError {
    #[error("workbook package could not be opened: {0}")]
    Io(#[from] std::io::Error),

    #[error("workbook package is not a readable zip archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("workbook package does not contain content.json")]
    MissingContent,

    #[error("workbook package uses an unsupported XMind format variant")]
    UnsupportedFormat,

    #[error("content.json could not be decoded: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageSheet {
    id: String,
    title: String,
    root_topic: StorageTopic,
    #[serde(flatten)]
    unknown_fields: Map<String, Value>,
}

impl StorageSheet {
    fn preserve_unknown_json_fields(&self, preservation: &mut PreservationBag, path: &str) {
        preserve_unknown_fields(preservation, path, &self.unknown_fields);
        self.root_topic
            .preserve_unknown_json_fields(preservation, &format!("{path}.rootTopic"));
    }
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
    #[serde(default)]
    title: Option<String>,
    notes: Option<StorageNotes>,
    href: Option<String>,
    image: Option<StorageImage>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    markers: Vec<StorageMarker>,
    #[serde(default)]
    children: StorageChildren,
    #[serde(flatten)]
    unknown_fields: Map<String, Value>,
}

impl StorageTopic {
    fn preserve_unknown_json_fields(&self, preservation: &mut PreservationBag, path: &str) {
        preserve_unknown_fields(preservation, path, &self.unknown_fields);
        for (index, child) in self.children.attached.iter().enumerate() {
            child.preserve_unknown_json_fields(
                preservation,
                &format!("{path}.children.attached[{index}]"),
            );
        }
    }
}

fn preserve_unknown_fields(
    preservation: &mut PreservationBag,
    path: &str,
    fields: &Map<String, Value>,
) {
    for (field, value) in fields {
        preservation.preserve_json_field(format!("{path}.{field}"), value.clone());
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageMarker {
    marker_id: String,
}

#[derive(Debug, Deserialize)]
struct StorageImage {
    src: Option<String>,
    alt: Option<String>,
    title: Option<String>,
}

impl StorageImage {
    fn into_topic_image(self) -> Option<TopicImageRef> {
        self.src
            .map(|src| TopicImageRef::new(AssetId::new(src), self.alt, self.title))
    }
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
            title: value.title.unwrap_or_default(),
            note: value
                .notes
                .and_then(|notes| notes.plain.map(|plain| plain.content)),
            labels: value.labels,
            markers: value
                .markers
                .into_iter()
                .map(|marker| marker.marker_id)
                .collect(),
            hyperlink: value.href,
            image: value.image.and_then(StorageImage::into_topic_image),
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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use zip::write::FileOptions;

    use super::read_workbook;

    #[test]
    fn read_workbook_preserves_unknown_package_entry_bytes() {
        let temp_dir = tempfile::tempdir().expect("temp dir is created");
        let workbook_path = temp_dir.path().join("unknown-package.xmind");
        let file = File::create(&workbook_path).expect("workbook file is created");
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("content.json", options)
            .expect("content entry starts");
        zip.write_all(
            br#"[{"id":"sheet-roadmap","title":"Roadmap","rootTopic":{"id":"topic-root","title":"Roadmap"}}]"#,
        )
        .expect("content entry is written");
        zip.start_file("metadata.json", options)
            .expect("metadata entry starts");
        zip.write_all(br#"{"vendor":true}"#)
            .expect("metadata entry is written");
        zip.finish().expect("zip finishes");

        let workbook = read_workbook(&workbook_path).expect("workbook reads");

        let entry = &workbook.preservation.package_entries()[0];
        assert_eq!(entry.name(), "metadata.json");
        assert_eq!(entry.bytes(), br#"{"vendor":true}"#);
    }
}
