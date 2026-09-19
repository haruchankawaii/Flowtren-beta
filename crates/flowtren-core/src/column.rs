use crate::semantic_type::SemanticType;

#[derive(Debug, Clone)]
pub struct ColumnProfile {
    pub name: String,
    pub semantic_type: SemanticType,
    pub null_count: usize,
    pub unique_count: usize,
}