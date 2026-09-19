use flowtren_quality::DatasetQuality;

use crate::{
    explanation::format_percentage,
    insight::{
        Insight,
        InsightConfidence,
        InsightEvidence,
        InsightKind,
        InsightSeverity,
    },
};

pub fn generate_duplicate_insight(
    quality: &DatasetQuality,
) -> Option<Insight> {
    if quality.row_count == 0
        || quality.duplicate_count == 0
    {
        return None;
    }

    let severity =
        severity_from_rate(
            quality.duplicate_rate,
        );

    let confidence =
        confidence_from_rows(
            quality.row_count,
        );

    let priority_score =
        (
            45.0
                + quality.duplicate_rate
                    * 100.0
        )
        .clamp(
            0.0,
            100.0,
        );

    let summary =
        format!(
            "{} duplicate rows were detected, representing {} of the dataset.",
            quality.duplicate_count,
            format_percentage(
                quality.duplicate_rate,
            ),
        );

    Some(
        Insight::new(
            InsightKind::DuplicateRows,

            "Duplicate rows detected",

            summary,

            Vec::new(),

            severity,
            confidence,
            priority_score,
        )
        .with_evidence(
            InsightEvidence::new(
                "Duplicate rows",
                quality
                    .duplicate_count
                    .to_string(),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Duplicate rate",
                format_percentage(
                    quality.duplicate_rate,
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Total rows",
                quality
                    .row_count
                    .to_string(),
            ),
        ),
    )
}

fn severity_from_rate(
    rate: f64,
) -> InsightSeverity {
    if rate >= 0.20 {
        InsightSeverity::High
    } else if rate >= 0.05 {
        InsightSeverity::Medium
    } else {
        InsightSeverity::Low
    }
}

fn confidence_from_rows(
    row_count: usize,
) -> InsightConfidence {
    if row_count >= 30 {
        InsightConfidence::High
    } else if row_count >= 10 {
        InsightConfidence::Medium
    } else {
        InsightConfidence::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use flowtren_quality::{
        QualitySeverity,
    };

    fn dataset_quality(
        duplicate_count: usize,
        duplicate_rate: f64,
    ) -> DatasetQuality {
        DatasetQuality {
            row_count:
                100,

            column_count:
                3,

            duplicate_count,

            duplicate_rate,

            duplicate_severity:
                QualitySeverity::Medium,

            columns:
                Vec::new(),

            issues:
                Vec::new(),

            score:
                90.0,
        }
    }

    #[test]
    fn generates_duplicate_insight() {
        let quality =
            dataset_quality(
                10,
                0.10,
            );

        let insight =
            generate_duplicate_insight(
                &quality,
            )
            .expect(
                "Duplicate insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::DuplicateRows
        );

        assert_eq!(
            insight.confidence,
            InsightConfidence::High
        );
    }

    #[test]
    fn no_duplicates_produce_no_insight() {
        let quality =
            dataset_quality(
                0,
                0.0,
            );

        assert!(
            generate_duplicate_insight(
                &quality,
            )
            .is_none()
        );
    }
}