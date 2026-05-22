use crate::domain::topic::Topic;

#[derive(Debug)]
pub struct Sheet {
    #[allow(dead_code)]
    pub id: SheetId,
    pub title: String,
    pub root: Topic,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct SheetId(pub String);
