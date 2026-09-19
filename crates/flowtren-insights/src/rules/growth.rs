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

const MIN_CHANGE_RATE: f64 =
    0.05;

pub fn generate_growth_insight(
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

    if change <= 0.0
        || change < MIN_CHANGE_RATE
    {
        return None;
    }

    let severity =
        severity_from_change(
            change,
        );

    let confidence =
        confidence_from_observations(
            observation_count,
        );

    let priority_score =
        (
            50.0
                + change * 50.0
        )
        .clamp(
            0.0,
            100.0,
        );

    let summary =
        format!(
            "'{column_name}' increased from {} to {}, a change of {}.",
            format_decimal(
                start_value,
            ),
            format_decimal(
                end_value,
            ),
            format_percentage(
                change,
            ),
        );

    Some(
        Insight::new(
            InsightKind::Growth,

            format!(
                "Growth detected in {column_name}"
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
                "Growth",
                format_percentage(
                    change,
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

pub(crate) fn calculate_change(
    start_value: f64,
    end_value: f64,
) -> Option<f64> {
    if !start_value.is_finite()
        || !end_value.is_finite()
    {
        return None;
    }

    if start_value == 0.0 {
        return None;
    }

    Some(
        (
            end_value
                - start_value
        )
            / start_value.abs(),
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
    fn generates_growth_insight() {
        let insight =
            generate_growth_insight(
                "sales",
                100.0,
                150.0,
                50,
            )
            .expect(
                "Growth insight should exist",
            );

        assert_eq!(
            insight.kind,
            InsightKind::Growth
        );

        assert_eq!(
            insight.confidence,
            InsightConfidence::High
        );
    }

    #[test]
    fn small_growth_is_ignored() {
        assert!(
            generate_growth_insight(
                "sales",
                100.0,
                102.0,
                100,
            )
            .is_none()
        );
    }

    #[test]
    fn decline_is_not_growth() {
        assert!(
            generate_growth_insight(
                "sales",
                100.0,
                80.0,
                100,
            )
            .is_none()
        );
    }

    #[test]
    fn zero_baseline_is_ignored() {
        assert!(
            generate_growth_insight(
                "sales",
                0.0,
                100.0,
                100,
            )
            .is_none()
        );
    }
}