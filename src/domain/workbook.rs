use crate::domain::sheet::Sheet;

#[derive(Debug)]
pub struct Workbook {
    pub sheets: Vec<Sheet>,
}
