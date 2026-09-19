use flowtren_quality::ColumnQuality;

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

pub fn generate_missing_insight(
    quality: &ColumnQuality,
) -> Option<Insight> {
    if quality.row_count == 0
        || quality.missing_count == 0
    {
        return None;
    }

    let severity =
        severity_from_rate(
            quality.missing_rate,
        );

    let confidence =
        confidence_from_rows(
            quality.row_count,
        );

    let priority_score =
        (
            40.0
                + quality.missing_rate
                    * 100.0
        )
        .clamp(
            0.0,
            100.0,
        );

    let summary =
        format!(
            "'{}' contains {} missing values, representing {} of its rows.",
            quality.name,
            quality.missing_count,
            format_percentage(
                quality.missing_rate,
            ),
        );

    Some(
        Insight::new(
            InsightKind::MissingValues,

            format!(
                "Missing values detected in {}",
                quality.name,
            ),

            summary,

            vec![
                quality.name.clone(),
            ],

            severity,
            confidence,
            priority_score,
        )
        .with_evidence(
            InsightEvidence::new(
                "Missing values",
                quality
                    .missing_count
                    .to_string(),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Missing rate",
                format_percentage(
                    quality.missing_rate,
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

    fn quality_with_missing()
        -> ColumnQuality
    {
        ColumnQuality {
            name:
                "email".to_string(),

            semantic_type:
                None,

            row_count:
                100,

            missing_count:
                10,

            missing_rate:
                0.10,

            missing_severity:
                QualitySeverity::Medium,

            unique_count:
                80,

            unique_rate:
                0.8888888889,

            inconsistent_count:
                0,

            inconsistency_rate:
                0.0,

            inconsistent_groups:
                0,

            issues:
                Vec::new(),

            score:
                94.0,
        }
    }

    #[test]
    fn generates_missing_insight() {
        let quality =
            quality_with_missing();

        let insight =
            generate_missing_insight(
                &quality,
            )
            .expect(
                "Missing insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::MissingValues
        );

        assert_eq!(
            insight.columns,
            vec![
                "email".to_string()
            ]
        );

        assert_eq!(
            insight.confidence,
            InsightConfidence::High
        );
    }

    #[test]
    fn clean_column_produces_no_insight() {
        let mut quality =
            quality_with_missing();

        quality.missing_count =
            0;

        quality.missing_rate =
            0.0;

        assert!(
            generate_missing_insight(
                &quality,
            )
            .is_none()
        );
    }
}