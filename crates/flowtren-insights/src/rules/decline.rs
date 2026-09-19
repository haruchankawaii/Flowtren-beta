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
    rules::growth::calculate_change,
};

const MIN_CHANGE_RATE: f64 =
    0.05;

pub fn generate_decline_insight(
    column_name: &str,
    start_value: f64,
    end_value: f64,
    observation_count: usize,
) -> Option<Insight> {
    let change =
        calculate_change(
            start_value,
            end_value,
        )?;

    if change >= 0.0 {
        return None;
    }

    let decline_rate =
        change.abs();

    if decline_rate
        < MIN_CHANGE_RATE
    {
        return None;
    }

    let severity =
        severity_from_change(
            decline_rate,
        );

    let confidence =
        confidence_from_observations(
            observation_count,
        );

    let priority_score =
        (
            50.0
                + decline_rate
                    * 50.0
        )
        .clamp(
            0.0,
            100.0,
        );

    let summary =
        format!(
            "'{column_name}' decreased from {} to {}, a decline of {}.",
            format_decimal(
                start_value,
            ),
            format_decimal(
                end_value,
            ),
            format_percentage(
                decline_rate,
            ),
        );

    Some(
        Insight::new(
            InsightKind::Decline,

            format!(
                "Decline detected in {column_name}"
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
                "Start value",
                format_decimal(
                    start_value,
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "End value",
                format_decimal(
                    end_value,
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Decline",
                format_percentage(
                    decline_rate,
                ),
            ),
        )
        .with_evidence(
            InsightEvidence::new(
                "Observations",
                observation_count
                    .to_string(),
            ),
        ),
    )
}

fn severity_from_change(
    rate: f64,
) -> InsightSeverity {
    if rate >= 0.50 {
        InsightSeverity::High
    } else if rate >= 0.20 {
        InsightSeverity::Medium
    } else {
        InsightSeverity::Low
    }
}

fn confidence_from_observations(
    count: usize,
) -> InsightConfidence {
    if count >= 30 {
        InsightConfidence::High
    } else if count >= 10 {
        InsightConfidence::Medium
    } else {
        InsightConfidence::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_decline_insight() {
        let insight =
            generate_decline_insight(
                "sales",
                100.0,
                70.0,
                50,
            )
            .expect(
                "Decline insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::Decline
        );

        assert_eq!(
            insight.confidence,
            InsightConfidence::High
        );
    }

    #[test]
    fn growth_is_not_decline() {
        assert!(
            generate_decline_insight(
                "sales",
                100.0,
                130.0,
                100,
            )
            .is_none()
        );
    }

    #[test]
    fn small_decline_is_ignored() {
        assert!(
            generate_decline_insight(
                "sales",
                100.0,
                98.0,
                100,
            )
            .is_none()
        );
    }

    #[test]
    fn zero_baseline_is_ignored() {
        assert!(
            generate_decline_insight(
                "sales",
                0.0,
                -10.0,
                100,
            )
            .is_none()
        );
    }
}