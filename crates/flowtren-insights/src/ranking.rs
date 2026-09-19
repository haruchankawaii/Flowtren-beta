use crate::insight::Insight;

pub fn rank_insights(
    insights: &mut [Insight],
) {
    insights.sort_by(
        |left, right| {
            right
                .priority_score
                .total_cmp(
                    &left.priority_score,
                )
                .then_with(
                    || {
                        right
                            .severity
                            .cmp(
                                &left.severity,
                            )
                    },
                )
                .then_with(
                    || {
                        left
                            .title
                            .cmp(
                                &right.title,
                            )
                    },
                )
        },
    );
}

pub fn ranked_insights(
    mut insights: Vec<Insight>,
) -> Vec<Insight> {
    rank_insights(
        &mut insights,
    );

    insights
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::insight::{
        InsightConfidence,
        InsightKind,
        InsightSeverity,
    };

    fn insight(
        title: &str,
        score: f64,
    ) -> Insight {
        Insight::new(
            InsightKind::Trend,
            title,
            "test",
            vec![],
            InsightSeverity::Medium,
            InsightConfidence::High,
            score,
        )
    }

    #[test]
    fn ranks_highest_score_first() {
        let insights =
            vec![
                insight(
                    "Low",
                    20.0,
                ),
                insight(
                    "High",
                    90.0,
                ),
                insight(
                    "Medium",
                    50.0,
                ),
            ];

        let ranked =
            ranked_insights(
                insights,
            );

        assert_eq!(
            ranked[0].title,
            "High"
        );

        assert_eq!(
            ranked[1].title,
            "Medium"
        );

        assert_eq!(
            ranked[2].title,
            "Low"
        );
    }
}