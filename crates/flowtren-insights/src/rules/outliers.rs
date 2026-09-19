use flowtren_stats::OutlierStats;

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

pub fn generate_outlier_insight(
    column_name: &str,
    stats: &OutlierStats,
) -> Option<Insight> {
    if stats.count == 0
        || stats.outlier_count == 0
    {
        return None;
    }

    let severity =
        severity_from_rate(
            stats.outlier_rate,
        );

    let confidence =
        if stats.count >= 30 {
            InsightConfidence::High
        } else if stats.count >= 10 {
            InsightConfidence::Medium
        } else {
            InsightConfidence::Low
        };

    let priority_score =
        (
            50.0
                + stats.outlier_rate
                    * 100.0
        )
        .clamp(
            0.0,
            100.0,
        );

    let summary =
        format!(
            "{} values in '{}' fall outside the expected IQR range ({} of valid values).",
            stats.outlier_count,
            column_name,
            format_percentage(
                stats.outlier_rate,
            ),
        );

    let mut insight =
        Insight::new(
            InsightKind::Outlier,

            format!(
                "Unusual values detected in {column_name}"
            ),

            summary,

            vec![
                column_name.to_string(),
            ],

            severity,
            confidence,
            priority_score,
        )
        .with_evidence(
            InsightEvidence::new(
                "Outlier count",
                stats
                    .outlier_count
                    .to_string(),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Outlier rate",
                format_percentage(
                    stats.outlier_rate,
                ),
            ),
        );

    if let Some(lower_bound) =
        stats.lower_bound
    {
        insight =
            insight.with_evidence(
                InsightEvidence::new(
                    "Lower bound",
                    format!(
                        "{lower_bound:.2}"
                    ),
                ),
            );
    }

    if let Some(upper_bound) =
        stats.upper_bound
    {
        insight =
            insight.with_evidence(
                InsightEvidence::new(
                    "Upper bound",
                    format!(
                        "{upper_bound:.2}"
                    ),
                ),
            );
    }

    Some(insight)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_outlier_insight() {
        let stats =
            OutlierStats {
                count: 100,

                q1:
                    Some(10.0),

                q3:
                    Some(20.0),

                iqr:
                    Some(10.0),

                lower_bound:
                    Some(-5.0),

                upper_bound:
                    Some(35.0),

                outlier_count:
                    5,

                outlier_rate:
                    0.05,

                lower_outlier_count:
                    2,

                upper_outlier_count:
                    3,
            };

        let insight =
            generate_outlier_insight(
                "revenue",
                &stats,
            )
            .expect(
                "Insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::Outlier
        );

        assert_eq!(
            insight.columns,
            vec![
                "revenue".to_string()
            ]
        );
    }

    #[test]
    fn no_outliers_produces_no_insight() {
        let stats =
            OutlierStats {
                count: 100,

                q1:
                    Some(10.0),

                q3:
                    Some(20.0),

                iqr:
                    Some(10.0),

                lower_bound:
                    Some(-5.0),

                upper_bound:
                    Some(35.0),

                outlier_count:
                    0,

                outlier_rate:
                    0.0,

                lower_outlier_count:
                    0,

                upper_outlier_count:
                    0,
            };

        assert!(
            generate_outlier_insight(
                "revenue",
                &stats,
            )
            .is_none()
        );
    }
}