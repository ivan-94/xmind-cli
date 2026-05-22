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

impl TopicImageRef {
    pub fn new(asset_id: AssetId, alt: Option<String>, title: Option<String>) -> Self {
        Self {
            asset_id,
            alt,
            title,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AssetId, TopicImageRef};

    #[test]
    fn asset_id_exposes_stable_string_identifier() {
        let asset_id = AssetId::new("xap:resources/payment.png");

        assert_eq!(asset_id.as_str(), "xap:resources/payment.png");
    }

    #[test]
    fn topic_image_ref_groups_asset_id_and_accessible_text() {
        let image = TopicImageRef::new(
            AssetId::new("xap:resources/payment.png"),
            Some("Payment flow diagram".to_owned()),
            Some("Payment flow".to_owned()),
        );

        assert_eq!(image.asset_id.as_str(), "xap:resources/payment.png");
        assert_eq!(image.alt.as_deref(), Some("Payment flow diagram"));
        assert_eq!(image.title.as_deref(), Some("Payment flow"));
    }
}
