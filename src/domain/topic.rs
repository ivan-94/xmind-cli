#[derive(Debug)]
pub struct Topic {
    pub id: TopicId,
    pub title: String,
    pub note: Option<String>,
    pub labels: Vec<String>,
    pub markers: Vec<String>,
    pub hyperlink: Option<String>,
    pub image: Option<TopicImageRef>,
    pub children: Vec<Topic>,
}

#[derive(Debug)]
pub struct TopicId(pub String);

#[derive(Debug)]
pub struct AssetId(pub String);

impl AssetId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct TopicImageRef {
    pub asset_id: AssetId,
    pub alt: Option<String>,
    pub title: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::AssetId;

    #[test]
    fn asset_id_exposes_stable_string_identifier() {
        let asset_id = AssetId::new("xap:resources/payment.png");

        assert_eq!(asset_id.as_str(), "xap:resources/payment.png");
    }
}
