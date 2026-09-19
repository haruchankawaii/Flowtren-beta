use flowtren_stats::DescriptiveStats;

use crate::{
    explanation::{
        format_decimal,
        format_percentage,
    },
    insight::{
        Insight,
        InsightConfidence,
        InsightEvidence,
        InsightKind,
        InsightSeverity,
    },
};

const MIN_VOLATILITY_RATE: f64 =
    0.30;

pub fn calculate_volatility(
    stats: &DescriptiveStats,
) -> Option<f64> {
    let mean =
        stats.mean?;

    let std_dev =
        stats.std_dev?;

    if mean == 0.0 {
        return None;
    }

    Some(
        (
            std_dev
                / mean.abs()
        )
        .max(0.0),
    )
}

pub fn generate_volatility_insight(
    column_name: &str,
    stats: &DescriptiveStats,
) -> Option<Insight> {
    let volatility =
        calculate_volatility(
            stats,
        )?;

    if volatility
        < MIN_VOLATILITY_RATE
    {
        return None;
    }

    let severity =
        if volatility >= 1.0 {
            InsightSeverity::High
        } else if volatility >= 0.60 {
            InsightSeverity::Medium
        } else {
            InsightSeverity::Low
        };

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
                + volatility
                    * 50.0
        )
        .clamp(
            0.0,
            100.0,
        );

    Some(
        Insight::new(
            InsightKind::Volatility,

            format!(
                "High variability detected in {column_name}"
            ),

            format!(
                "'{}' varies substantially relative to its average value.",
                column_name,
            ),

            vec![
                column_name.to_string(),
            ],

            severity,
            confidence,
            priority_score,
        )
        .with_evidence(
            InsightEvidence::new(
                "Mean",
                format_decimal(
                    stats.mean.unwrap(),
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Standard deviation",
                format_decimal(
                    stats.std_dev.unwrap(),
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Coefficient of variation",
                format_percentage(
                    volatility,
                ),
            ),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(
        mean: f64,
        std_dev: f64,
        count: usize,
    ) -> DescriptiveStats {
        DescriptiveStats {
            row_count:
                count,

            count,

            null_count:
                0,

            sum:
                Some(
                    mean
                        * count as f64,
                ),

            mean:
                Some(mean),

            min:
                Some(0.0),

            max:
                Some(
                    mean * 2.0,
                ),

            std_dev:
                Some(std_dev),
        }
    }

    #[test]
    fn calculates_volatility() {
        let descriptive =
            stats(
                100.0,
                50.0,
                100,
            );

        assert_eq!(
            calculate_volatility(
                &descriptive,
            ),
            Some(0.5)
        );
    }

    #[test]
    fn generates_volatility_insight() {
        let descriptive =
            stats(
                100.0,
                80.0,
                100,
            );

        let insight =
            generate_volatility_insight(
                "revenue",
                &descriptive,
            )
            .expect(
                "Volatility insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::Volatility
        );

        assert_eq!(
            insight.confidence,
            InsightConfidence::High
        );
    }

    #[test]
    fn stable_column_is_ignored() {
        let descriptive =
            stats(
                100.0,
                10.0,
                100,
            );

        assert!(
            generate_volatility_insight(
                "revenue",
                &descriptive,
            )
            .is_none()
        );
    }

    #[test]
    fn zero_mean_is_ignored() {
        let descriptive =
            stats(
                0.0,
                10.0,
                100,
            );

        assert!(
            generate_volatility_insight(
                "revenue",
                &descriptive,
            )
            .is_none()
        );
    }
}