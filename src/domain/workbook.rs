use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::domain::sheet::Sheet;
use crate::domain::topic::AssetId;

#[derive(Debug)]
pub struct Workbook {
    pub sheets: Vec<Sheet>,
    pub resources: ResourceIndex,
    pub preservation: PreservationBag,
}

#[derive(Debug, Default)]
pub struct ResourceIndex {
    assets: BTreeMap<AssetId, ()>,
}

#[derive(Debug, Default)]
pub struct PreservationBag {
    raw_json_fields: Map<String, Value>,
    package_entries: Vec<PreservedPackageEntry>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PreservedPackageEntry {
    name: String,
    bytes: Vec<u8>,
}

#[allow(dead_code)]
impl PreservedPackageEntry {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl PreservationBag {
    pub fn is_empty(&self) -> bool {
        self.raw_json_fields.is_empty() && self.package_entries.is_empty()
    }

    pub fn preserve_json_field(&mut self, field: impl Into<String>, value: Value) {
        self.raw_json_fields.insert(field.into(), value);
    }

    #[allow(dead_code)]
    pub fn preserve_package_entry(&mut self, entry_name: impl Into<String>) {
        self.preserve_package_entry_bytes(entry_name, Vec::new());
    }

    pub fn preserve_package_entry_bytes(&mut self, entry_name: impl Into<String>, bytes: Vec<u8>) {
        self.package_entries.push(PreservedPackageEntry {
            name: entry_name.into(),
            bytes,
        });
    }

    pub fn raw_json_fields(&self) -> &Map<String, Value> {
        &self.raw_json_fields
    }

    #[allow(dead_code)]
    pub fn package_entries(&self) -> &[PreservedPackageEntry] {
        &self.package_entries
    }

    #[cfg(test)]
    fn raw_json_field_count(&self) -> usize {
        self.raw_json_fields.len()
    }

    #[cfg(test)]
    fn package_entry_count(&self) -> usize {
        self.package_entries.len()
    }
}

impl ResourceIndex {
    pub fn insert_asset_id(&mut self, asset_id: AssetId) {
        self.assets.insert(asset_id, ());
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn asset_ids(&self) -> Vec<String> {
        self.assets
            .keys()
            .map(|asset_id| asset_id.as_str().to_owned())
            .collect()
    }

    #[cfg(test)]
    fn contains_asset_id(&self, asset_id: &AssetId) -> bool {
        self.assets.contains_key(asset_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::topic::AssetId;

    use super::{PreservationBag, ResourceIndex};

    #[test]
    fn resource_index_tracks_assets_by_id() {
        let mut index = ResourceIndex::default();

        index.insert_asset_id(AssetId::new("xap:resources/payment.png"));

        assert_eq!(index.len(), 1);
        assert!(index.contains_asset_id(&AssetId::new("xap:resources/payment.png")));
    }

    #[test]
    fn preservation_bag_tracks_unknown_json_fields_and_package_entries() {
        let mut bag = PreservationBag::default();

        bag.preserve_json_field("extensions", serde_json::json!({ "vendor": true }));
        bag.preserve_package_entry("metadata.json");

        assert!(!bag.is_empty());
        assert_eq!(bag.raw_json_field_count(), 1);
        assert_eq!(bag.package_entry_count(), 1);
    }

    #[test]
    fn preservation_bag_keeps_package_entry_bytes_for_reuse() {
        let mut bag = PreservationBag::default();

        bag.preserve_package_entry_bytes("metadata.json", br#"{"vendor":true}"#.to_vec());

        let entry = &bag.package_entries()[0];
        assert_eq!(entry.name(), "metadata.json");
        assert_eq!(entry.bytes(), br#"{"vendor":true}"#);
    }
}
