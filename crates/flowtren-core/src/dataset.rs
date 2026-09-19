use crate::column::ColumnProfile;

#[derive(Debug, Clone)]
pub struct DatasetProfile {
    pub row_count: usize,
    pub column_count: usize,
    pub columns: Vec<ColumnProfile>,
}