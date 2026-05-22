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

#[derive(Debug)]
pub struct TopicImageRef {
    pub asset_id: AssetId,
    pub alt: Option<String>,
    pub title: Option<String>,
}
