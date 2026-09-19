use std::collections::HashSet;

use crate::{
    dataset_quality::DatasetQuality,
    issue::{
        QualityIssue,
        QualityIssueKind,
    },
};

#[derive(Debug, Clone)]
pub struct QualityComparison {
    pub before_score:
        f64,

    pub after_score:
        f64,

    pub score_change:
        f64,

    pub improved:
        bool,

    pub before_issue_count:
        usize,

    pub after_issue_count:
        usize,

    pub resolved_issue_count:
        usize,

    pub new_issue_count:
        usize,

    pub remaining_issue_count:
        usize,
}

pub fn compare_quality(
    before: &DatasetQuality,
    after: &DatasetQuality,
) -> QualityComparison {
    let before_keys =
        issue_keys(
            &before.issues,
        );

    let after_keys =
        issue_keys(
            &after.issues,
        );

    let resolved_issue_count =
        before_keys
            .difference(
                &after_keys,
            )
            .count();

    let new_issue_count =
        after_keys
            .difference(
                &before_keys,
            )
            .count();

    let remaining_issue_count =
        before_keys
            .intersection(
                &after_keys,
            )
            .count();

    let score_change =
        after.score
            - before.score;

    QualityComparison {
        before_score:
            before.score,

        after_score:
            after.score,

        score_change,

        improved:
            score_change > 0.0,

        before_issue_count:
            before.issues.len(),

        after_issue_count:
            after.issues.len(),

        resolved_issue_count,
        new_issue_count,
        remaining_issue_count,
    }
}

fn issue_keys(
    issues: &[QualityIssue],
) -> HashSet<(
    QualityIssueKind,
    Option<String>,
)> {
    issues
        .iter()
        .map(
            |issue| {
                (
                    issue.kind.clone(),
                    issue.column.clone(),
                )
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dataset_quality::analyze_dataset_quality;

    use polars::prelude::*;

    #[test]
    fn detects_quality_improvement() {
        let before =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            Some(1_i64),
                            Some(2_i64),
                            Some(3_i64),
                        ],
                    ),

                    Column::new(
                        "city".into(),
                        vec![
                            Some("Jakarta"),
                            None,
                            Some("Bandung"),
                        ],
                    ),
                ],
            )
            .expect(
                "Before dataframe should build",
            );

        let after =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            2_i64,
                            3_i64,
                        ],
                    ),

                    Column::new(
                        "city".into(),
                        vec![
                            "Jakarta",
                            "Surabaya",
                            "Bandung",
                        ],
                    ),
                ],
            )
            .expect(
                "After dataframe should build",
            );

        let before_quality =
            analyze_dataset_quality(
                &before,
            )
            .expect(
                "Before quality should succeed",
            );

        let after_quality =
            analyze_dataset_quality(
                &after,
            )
            .expect(
                "After quality should succeed",
            );

        let comparison =
            compare_quality(
                &before_quality,
                &after_quality,
            );

        assert!(
            comparison.improved
        );

        assert!(
            comparison.after_score
                > comparison.before_score
        );

        assert_eq!(
            comparison.resolved_issue_count,
            1
        );

        assert_eq!(
            comparison.new_issue_count,
            0
        );
    }

    #[test]
    fn perfect_to_perfect_has_no_change() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            2_i64,
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let quality =
            analyze_dataset_quality(
                &df,
            )
            .expect(
                "Quality should succeed",
            );

        let comparison =
            compare_quality(
                &quality,
                &quality,
            );

        assert_eq!(
            comparison.score_change,
            0.0
        );

        assert!(
            !comparison.improved
        );

        assert_eq!(
            comparison.resolved_issue_count,
            0
        );

        assert_eq!(
            comparison.new_issue_count,
            0
        );
    }
}