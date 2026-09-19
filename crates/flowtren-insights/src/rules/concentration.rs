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

const MIN_CONCENTRATION_SCORE: f64 =
    0.35;

pub fn calculate_concentration(
    counts: &[usize],
) -> Option<f64> {
    let total:
        usize =
        counts.iter().sum();

    if total == 0 {
        return None;
    }

    let score =
        counts
            .iter()
            .map(|count| {
                let share =
                    *count as f64
                        / total as f64;

                share * share
            })
            .sum();

    Some(score)
}

pub fn generate_concentration_insight(
    column_name: &str,
    counts: &[usize],
) -> Option<Insight> {
    let concentration =
        calculate_concentration(
            counts,
        )?;

    if concentration
        < MIN_CONCENTRATION_SCORE
    {
        return None;
    }

    let total:
        usize =
        counts.iter().sum();

    let severity =
        if concentration >= 0.70 {
            InsightSeverity::High
        } else if concentration >= 0.50 {
            InsightSeverity::Medium
        } else {
            InsightSeverity::Low
        };

    let confidence =
        if total >= 30 {
            InsightConfidence::High
        } else if total >= 10 {
            InsightConfidence::Medium
        } else {
            InsightConfidence::Low
        };

    let priority_score =
        concentration * 100.0;

    Some(
        Insight::new(
            InsightKind::Concentration,

            format!(
                "High concentration detected in {column_name}"
            ),

            format!(
                "Values in '{}' are concentrated among a relatively small number of categories.",
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
                "Concentration score",
                format_decimal(
                    concentration,
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Observed values",
                total.to_string(),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Categories",
                counts
                    .len()
                    .to_string(),
            ),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_concentration() {
        let score =
            calculate_concentration(
                &[
                    50,
                    50,
                ],
            )
            .expect(
                "Score should exist",
            );

        assert_eq!(
            score,
            0.5
        );
    }

    #[test]
    fn concentrated_distribution_generates_insight() {
        let insight =
            generate_concentration_insight(
                "category",
                &[
                    80,
                    10,
                    5,
                    5,
                ],
            )
            .expect(
                "Concentration insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::Concentration
        );
    }

    #[test]
    fn balanced_distribution_is_ignored() {
        assert!(
            generate_concentration_insight(
                "category",
                &[
                    25,
                    25,
                    25,
                    25,
                ],
            )
            .is_none()
        );
    }

    #[test]
    fn empty_counts_are_ignored() {
        assert!(
            generate_concentration_insight(
                "category",
                &[],
            )
            .is_none()
        );
    }
}