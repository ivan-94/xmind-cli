#[derive(Debug)]
pub struct Topic {
    pub id: TopicId,
    pub title: String,
    pub note: Option<String>,
    pub labels: Vec<String>,
    pub children: Vec<Topic>,
}

#[derive(Debug)]
pub struct TopicId(pub String);
