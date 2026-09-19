use crate::severity::QualitySeverity;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
)]
pub enum QualityIssueKind {
    MissingValues,
    DuplicateRows,
    InconsistentValues,

    // Semantic-aware issues
    IdentifierNotUnique,
    CategoryCardinalityHigh,
}

#[derive(Debug, Clone)]
pub struct QualityIssue {
    pub kind: QualityIssueKind,

    pub column:
        Option<String>,

    pub affected_rows:
        usize,

    pub rate:
        f64,

    pub severity:
        QualitySeverity,
}

impl QualityIssue {
    pub fn new(
        kind: QualityIssueKind,
        column: Option<String>,
        affected_rows: usize,
        rate: f64,
        severity: QualitySeverity,
    ) -> Self {
        Self {
            kind,
            column,
            affected_rows,

            rate:
                rate.clamp(
                    0.0,
                    1.0,
                ),

            severity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_issue() {
        let issue =
            QualityIssue::new(
                QualityIssueKind::MissingValues,

                Some(
                    "email".to_string(),
                ),

                10,

                0.25,

                QualitySeverity::High,
            );

        assert_eq!(
            issue.kind,
            QualityIssueKind::MissingValues
        );

        assert_eq!(
            issue.column.as_deref(),
            Some("email")
        );

        assert_eq!(
            issue.affected_rows,
            10
        );

        assert_eq!(
            issue.rate,
            0.25
        );
    }

    #[test]
    fn clamps_rate() {
        let issue =
            QualityIssue::new(
                QualityIssueKind::DuplicateRows,
                None,
                1,
                10.0,
                QualitySeverity::High,
            );

        assert_eq!(
            issue.rate,
            1.0
        );
    }

    #[test]
    fn semantic_issue_kind_can_be_compared() {
        let left =
            QualityIssueKind::IdentifierNotUnique;

        let right =
            QualityIssueKind::IdentifierNotUnique;

        assert_eq!(
            left,
            right
        );
    }
}