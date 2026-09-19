use std::collections::HashMap;

use flowtren_core::semantic_type::SemanticType;
use polars::prelude::*;

use crate::{
    column_quality::{
        ColumnQuality,
        analyze_column_quality,
    },
    duplicates::{
        DuplicateMetrics,
        analyze_duplicates,
    },
    issue::{
        QualityIssue,
        QualityIssueKind,
    },
    score::{
        combine_scores,
        score_from_duplicate_rate,
        weighted_score,
    },
    semantic::analyze_semantic_quality,
    severity::{
        QualitySeverity,
        severity_from_rate,
    },
};

#[derive(Debug, Clone)]
pub struct DatasetQuality {
    pub row_count:
        usize,

    pub column_count:
        usize,

    pub duplicate_count:
        usize,

    pub duplicate_rate:
        f64,

    pub duplicate_severity:
        QualitySeverity,

    pub columns:
        Vec<ColumnQuality>,

    pub issues:
        Vec<QualityIssue>,

    pub score:
        f64,
}

pub fn analyze_dataset_quality(
    df: &DataFrame,
) -> PolarsResult<DatasetQuality> {
    let semantics:
        HashMap<String, SemanticType> =
        HashMap::new();

    analyze_dataset_quality_with_semantics(
        df,
        &semantics,
    )
}

pub fn analyze_dataset_quality_with_semantics(
    df: &DataFrame,
    semantics: &HashMap<String, SemanticType>,
) -> PolarsResult<DatasetQuality> {
    let DuplicateMetrics {
        duplicate_count,
        duplicate_rate,
    } =
        analyze_duplicates(
            df,
        )?;

    let mut columns =
        Vec::with_capacity(
            df.width(),
        );

    let mut column_scores =
        Vec::with_capacity(
            df.width(),
        );

    let mut issues =
        Vec::new();

    for column in df.columns() {
        let series =
            column
                .as_materialized_series();

        let mut quality =
            analyze_column_quality(
                series,
            )?;

        // ----------------------------------
        // Semantic-aware scoring
        // ----------------------------------
        if let Some(
            semantic_type,
        ) =
            semantics.get(
                quality.name.as_str(),
            )
        {
            let semantic_quality =
                analyze_semantic_quality(
                    &quality,
                    semantic_type,
                );

            // Base quality:
            // missing + consistency
            //
            // Semantic:
            // identifier uniqueness,
            // category cardinality, etc.
            quality.score =
                weighted_score(
                    &[
                        (
                            quality.score,
                            0.85,
                        ),
                        (
                            semantic_quality.score,
                            0.15,
                        ),
                    ],
                );

            quality.semantic_type =
                Some(
                    semantic_type.clone(),
                );

            quality
                .issues
                .extend(
                    semantic_quality
                        .issues,
                );
        }

        column_scores.push(
            quality.score,
        );

        issues.extend(
            quality
                .issues
                .iter()
                .cloned(),
        );

        columns.push(
            quality,
        );
    }

    if duplicate_count > 0 {
        issues.push(
            QualityIssue::new(
                QualityIssueKind::DuplicateRows,

                None,

                duplicate_count,

                duplicate_rate,

                severity_from_rate(
                    duplicate_rate,
                ),
            ),
        );
    }

    let average_column_score =
        combine_scores(
            &column_scores,
        );

    let duplicate_score =
        score_from_duplicate_rate(
            duplicate_rate,
        );

    let score =
        weighted_score(
            &[
                (
                    average_column_score,
                    0.80,
                ),
                (
                    duplicate_score,
                    0.20,
                ),
            ],
        );

    Ok(
        DatasetQuality {
            row_count:
                df.height(),

            column_count:
                df.width(),

            duplicate_count,
            duplicate_rate,

            duplicate_severity:
                severity_from_rate(
                    duplicate_rate,
                ),

            columns,
            issues,

            score,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_dataset_quality() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            1_i64,
                            2_i64,
                            3_i64,
                        ],
                    ),

                    Column::new(
                        "city".into(),
                        vec![
                            Some("Jakarta"),
                            Some("Jakarta"),
                            Some("jakarta"),
                            None,
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
                "Dataset quality should succeed",
            );

        assert_eq!(
            quality.row_count,
            4
        );

        assert_eq!(
            quality.column_count,
            2
        );

        assert_eq!(
            quality.duplicate_count,
            1
        );

        assert_eq!(
            quality.columns.len(),
            2
        );

        assert!(
            !quality
                .issues
                .is_empty()
        );

        assert!(
            quality.score
                >= 0.0
        );

        assert!(
            quality.score
                <= 100.0
        );
    }

    #[test]
    fn perfect_dataset_scores_100() {
        let df =
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
                            "Bandung",
                            "Surabaya",
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
                "Dataset quality should succeed",
            );

        assert_eq!(
            quality.score,
            100.0
        );

        assert!(
            quality.issues.is_empty()
        );
    }

    #[test]
    fn semantic_identifier_check_is_applied() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "customer_id".into(),
                        vec![
                            "C001",
                            "C001",
                            "C002",
                            "C003",
                        ],
                    ),

                    Column::new(
                        "name".into(),
                        vec![
                            "Alice",
                            "Bob",
                            "Charlie",
                            "David",
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let semantics =
            HashMap::from([
                (
                    "customer_id"
                        .to_string(),

                    SemanticType::Identifier,
                ),
            ]);

        let quality =
            analyze_dataset_quality_with_semantics(
                &df,
                &semantics,
            )
            .expect(
                "Semantic quality should succeed",
            );

        assert!(
            quality
                .issues
                .iter()
                .any(
                    |issue| {
                        issue.kind
                            == QualityIssueKind::IdentifierNotUnique
                    },
                )
        );

        let id_quality =
            quality
                .columns
                .iter()
                .find(
                    |column| {
                        column.name
                            == "customer_id"
                    },
                )
                .expect(
                    "customer_id should exist",
                );

        assert_eq!(
            id_quality.semantic_type,
            Some(
                SemanticType::Identifier
            )
        );

        assert!(
            id_quality.score
                < 100.0
        );
    }
}