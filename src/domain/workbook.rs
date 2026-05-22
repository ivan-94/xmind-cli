use std::collections::BTreeMap;

use crate::domain::sheet::Sheet;
use crate::domain::topic::AssetId;

#[derive(Debug)]
pub struct Workbook {
    pub sheets: Vec<Sheet>,
    pub resources: ResourceIndex,
}

#[derive(Debug, Default)]
pub struct ResourceIndex {
    assets: BTreeMap<AssetId, ()>,
}

impl ResourceIndex {
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    #[cfg(test)]
    fn insert_asset_id(&mut self, asset_id: AssetId) {
        self.assets.insert(asset_id, ());
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::topic::AssetId;

    use super::ResourceIndex;

    #[test]
    fn resource_index_tracks_assets_by_id() {
        let mut index = ResourceIndex::default();

        index.insert_asset_id(AssetId::new("xap:resources/payment.png"));

        assert_eq!(index.len(), 1);
    }
}
