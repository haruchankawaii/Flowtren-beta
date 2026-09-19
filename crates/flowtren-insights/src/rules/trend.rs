use flowtren_stats::{
    TrendDirection,
    TrendStats,
};

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

const MIN_R_SQUARED: f64 =
    0.50;

pub fn generate_trend_insight(
    column_name: &str,
    stats: &TrendStats,
) -> Option<Insight> {
    let direction =
        stats.direction?;

    let slope =
        stats.slope?;

    // Flat tidak kita tampilkan sebagai
    // insight utama untuk sekarang.
    if direction
        == TrendDirection::Flat
    {
        return None;
    }

    let r_squared =
        stats.r_squared?;

    if r_squared
        < MIN_R_SQUARED
    {
        return None;
    }

    let direction_text =
        match direction {
            TrendDirection::Increasing => {
                "increasing"
            }

            TrendDirection::Decreasing => {
                "decreasing"
            }

            TrendDirection::Flat => {
                return None;
            }
        };

    let confidence =
        if r_squared >= 0.80
            && stats.pair_count >= 10
        {
            InsightConfidence::High
        } else if r_squared >= 0.60 {
            InsightConfidence::Medium
        } else {
            InsightConfidence::Low
        };

    let severity =
        if r_squared >= 0.80 {
            InsightSeverity::High
        } else {
            InsightSeverity::Medium
        };

    let priority_score =
        (
            r_squared
                * 100.0
        )
        .clamp(
            0.0,
            100.0,
        );

    let summary =
        format!(
            "'{column_name}' shows an {direction_text} trend with slope {} and R² {}.",
            format_decimal(
                slope,
            ),
            format_decimal(
                r_squared,
            ),
        );

    Some(
        Insight::new(
            InsightKind::Trend,

            format!(
                "{} trend detected in {column_name}",
                capitalize(
                    direction_text,
                ),
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
                "Slope",
                format_decimal(
                    slope,
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "R²",
                format_decimal(
                    r_squared,
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Observations",
                stats
                    .pair_count
                    .to_string(),
            ),
        ),
    )
}

fn capitalize(
    value: &str,
) -> String {
    let mut chars =
        value.chars();

    let Some(first) =
        chars.next()
    else {
        return String::new();
    };

    format!(
        "{}{}",
        first
            .to_uppercase()
            .collect::<String>(),
        chars.collect::<String>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_increasing_trend() {
        let stats =
            TrendStats {
                pair_count: 20,

                slope:
                    Some(10.0),

                intercept:
                    Some(0.0),

                r_squared:
                    Some(0.90),

                direction:
                    Some(
                        TrendDirection::Increasing,
                    ),
            };

        let insight =
            generate_trend_insight(
                "sales",
                &stats,
            )
            .expect(
                "Trend insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::Trend
        );

        assert_eq!(
            insight.confidence,
            InsightConfidence::High
        );
    }

    #[test]
    fn weak_trend_is_ignored() {
        let stats =
            TrendStats {
                pair_count: 20,

                slope:
                    Some(2.0),

                intercept:
                    Some(0.0),

                r_squared:
                    Some(0.20),

                direction:
                    Some(
                        TrendDirection::Increasing,
                    ),
            };

        assert!(
            generate_trend_insight(
                "sales",
                &stats,
            )
            .is_none()
        );
    }

    #[test]
    fn flat_trend_is_ignored() {
        let stats =
            TrendStats {
                pair_count: 20,

                slope:
                    Some(0.0),

                intercept:
                    Some(10.0),

                r_squared:
                    None,

                direction:
                    Some(
                        TrendDirection::Flat,
                    ),
            };

        assert!(
            generate_trend_insight(
                "sales",
                &stats,
            )
            .is_none()
        );
    }
}