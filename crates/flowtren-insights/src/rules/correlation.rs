use flowtren_stats::CorrelationStats;

use crate::{
    explanation::format_decimal,
    insight::{
        Insight,
        InsightConfidence,
        InsightEvidence,
        InsightKind,
        InsightSeverity,
    },
};

const MIN_CORRELATION_STRENGTH: f64 =
    0.70;

pub fn generate_correlation_insight(
    left_column: &str,
    right_column: &str,
    stats: &CorrelationStats,
) -> Option<Insight> {
    let correlation =
        stats.pearson?;

    let strength =
        correlation.abs();

    if strength
        < MIN_CORRELATION_STRENGTH
    {
        return None;
    }

    let relationship =
        if correlation > 0.0 {
            "positive"
        } else {
            "negative"
        };

    let strength_label =
        if strength >= 0.90 {
            "Very strong"
        } else if strength >= 0.80 {
            "Strong"
        } else {
            "Moderately strong"
        };

    let severity =
        if strength >= 0.90 {
            InsightSeverity::High
        } else {
            InsightSeverity::Medium
        };

    let confidence =
        if stats.pair_count >= 30 {
            InsightConfidence::High
        } else if stats.pair_count >= 10 {
            InsightConfidence::Medium
        } else {
            InsightConfidence::Low
        };

    let priority_score =
        strength
            * 100.0;

    let summary =
        format!(
            "'{left_column}' and '{right_column}' show a {relationship} linear relationship with Pearson correlation {}.",
            format_decimal(
                correlation,
            ),
        );

    Some(
        Insight::new(
            InsightKind::Correlation,

            format!(
                "{strength_label} {relationship} correlation"
            ),

            summary,

            vec![
                left_column.to_string(),
                right_column.to_string(),
            ],

            severity,
            confidence,
            priority_score,
        )
        .with_evidence(
            InsightEvidence::new(
                "Pearson correlation",
                format_decimal(
                    correlation,
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Valid pairs",
                stats
                    .pair_count
                    .to_string(),
            ),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_positive_correlation() {
        let stats =
            CorrelationStats {
                pair_count: 100,
                pearson:
                    Some(0.92),
            };

        let insight =
            generate_correlation_insight(
                "sales",
                "marketing",
                &stats,
            )
            .expect(
                "Insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::Correlation
        );

        assert_eq!(
            insight.confidence,
            InsightConfidence::High
        );
    }

    #[test]
    fn generates_negative_correlation() {
        let stats =
            CorrelationStats {
                pair_count: 50,
                pearson:
                    Some(-0.85),
            };

        let insight =
            generate_correlation_insight(
                "price",
                "demand",
                &stats,
            )
            .expect(
                "Insight should exist",
            );

        assert!(
            insight.summary
                .contains(
                    "negative",
                )
        );
    }

    #[test]
    fn weak_correlation_is_ignored() {
        let stats =
            CorrelationStats {
                pair_count: 100,
                pearson:
                    Some(0.30),
            };

        assert!(
            generate_correlation_insight(
                "x",
                "y",
                &stats,
            )
            .is_none()
        );
    }
}