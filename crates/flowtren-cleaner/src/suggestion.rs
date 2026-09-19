#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleaningKind {
    TrimWhitespace,
    EmptyStringToNull,
    NormalizeMissingMarker,
    RemoveDuplicateRows,
    NormalizeNumberFormat,
    NormalizeCurrency,
    NormalizePercentage,
    NormalizeDate,
    RemoveEmptyRows,
    NormalizeCategories,
    NormalizeCasing,
}

#[derive(Debug, Clone)]
pub struct CleaningSuggestion {
    pub column: String,
    pub kind: CleaningKind,
    pub affected_rows: usize,
    pub confidence: f32,
}

impl CleaningSuggestion {
    pub fn new(
        column: impl Into<String>,
        kind: CleaningKind,
        affected_rows: usize,
        confidence: f32,
    ) -> Self {
        Self {
            column: column.into(),
            kind,
            affected_rows,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}