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

const MIN_DOMINANCE_RATE: f64 = 0.60;

pub fn generate_dominance_insight(
    column_name: &str,
    dominant_value: &str,
    dominant_count: usize,
    total_count: usize,
) -> Option<Insight> {
    if total_count == 0
        || dominant_count == 0
        || dominant_count > total_count
    {
        return None;
    }

    let dominance_rate =
        dominant_count as f64
            / total_count as f64;

    if dominance_rate < MIN_DOMINANCE_RATE {
        return None;
    }

    let severity =
        if dominance_rate >= 0.90 {
            InsightSeverity::High
        } else if dominance_rate >= 0.75 {
            InsightSeverity::Medium
        } else {
            InsightSeverity::Low
        };

    let confidence =
        if total_count >= 30 {
            InsightConfidence::High
        } else if total_count >= 10 {
            InsightConfidence::Medium
        } else {
            InsightConfidence::Low
        };

    let priority_score =
        dominance_rate * 100.0;

    Some(
        Insight::new(
            InsightKind::Dominance,

            format!(
                "'{dominant_value}' dominates {column_name}"
            ),

            format!(
                "'{dominant_value}' appears in {} of the observed values in '{}'.",
                format_percentage(
                    dominance_rate,
                ),
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
                "Dominant value",
                dominant_value,
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Occurrences",
                dominant_count
                    .to_string(),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Share",
                format_percentage(
                    dominance_rate,
                ),
            ),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_dominance_insight() {
        let insight =
            generate_dominance_insight(
                "city",
                "Jakarta",
                80,
                100,
            )
            .expect(
                "Dominance insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::Dominance
        );

        assert_eq!(
            insight.confidence,
            InsightConfidence::High
        );
    }

    #[test]
    fn weak_dominance_is_ignored() {
        assert!(
            generate_dominance_insight(
                "city",
                "Jakarta",
                40,
                100,
            )
            .is_none()
        );
    }

    #[test]
    fn empty_input_is_ignored() {
        assert!(
            generate_dominance_insight(
                "city",
                "Jakarta",
                0,
                0,
            )
            .is_none()
        );
    }
}