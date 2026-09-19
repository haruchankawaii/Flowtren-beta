#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum InsightKind {
    Growth,
    Decline,
    Dominance,
    Outlier,
    MissingValues,
    DuplicateRows,
    Correlation,
    Volatility,
    Concentration,
    Trend,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum InsightSeverity {
    Info,
    Low,
    Medium,
    High,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum InsightConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsightEvidence {
    pub label: String,
    pub value: String,
}

impl InsightEvidence {
    pub fn new(
        label: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Insight {
    pub kind: InsightKind,

    pub title: String,

    pub summary: String,

    pub columns: Vec<String>,

    pub severity: InsightSeverity,

    pub confidence: InsightConfidence,

    /// 0-100.
    ///
    /// Dipakai untuk menentukan urutan insight
    /// yang paling layak ditampilkan ke user.
    pub priority_score: f64,

    pub evidence: Vec<InsightEvidence>,
}

impl Insight {
    pub fn new(
        kind: InsightKind,
        title: impl Into<String>,
        summary: impl Into<String>,
        columns: Vec<String>,
        severity: InsightSeverity,
        confidence: InsightConfidence,
        priority_score: f64,
    ) -> Self {
        Self {
            kind,

            title:
                title.into(),

            summary:
                summary.into(),

            columns,

            severity,
            confidence,

            priority_score:
                priority_score.clamp(
                    0.0,
                    100.0,
                ),

            evidence:
                Vec::new(),
        }
    }

    pub fn with_evidence(
        mut self,
        evidence: InsightEvidence,
    ) -> Self {
        self.evidence.push(
            evidence,
        );

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_insight() {
        let insight =
            Insight::new(
                InsightKind::Trend,
                "Increasing trend",
                "Sales are increasing.",
                vec![
                    "sales".to_string(),
                ],
                InsightSeverity::Medium,
                InsightConfidence::High,
                85.0,
            );

        assert_eq!(
            insight.kind,
            InsightKind::Trend
        );

        assert_eq!(
            insight.priority_score,
            85.0
        );
    }

    #[test]
    fn clamps_priority_score() {
        let insight =
            Insight::new(
                InsightKind::Trend,
                "Test",
                "Test",
                vec![],
                InsightSeverity::Info,
                InsightConfidence::Low,
                150.0,
            );

        assert_eq!(
            insight.priority_score,
            100.0
        );
    }

    #[test]
    fn adds_evidence() {
        let insight =
            Insight::new(
                InsightKind::Correlation,
                "Correlation",
                "Test",
                vec![],
                InsightSeverity::Medium,
                InsightConfidence::High,
                80.0,
            )
            .with_evidence(
                InsightEvidence::new(
                    "Correlation",
                    "0.91",
                ),
            );

        assert_eq!(
            insight.evidence.len(),
            1
        );
    }
}