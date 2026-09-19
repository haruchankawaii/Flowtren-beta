use flowtren_core::semantic_type::SemanticType;

use crate::{
    column_quality::ColumnQuality,
    issue::{
        QualityIssue,
        QualityIssueKind,
    },
    score::score_from_rate,
    severity::severity_from_rate,
};

const IDENTIFIER_MIN_UNIQUE_RATE: f64 =
    0.95;

const CATEGORY_MAX_UNIQUE_RATE: f64 =
    0.50;

const CATEGORY_MIN_ROWS: usize =
    20;

#[derive(Debug, Clone)]
pub struct SemanticQuality {
    pub score: f64,
    pub issues: Vec<QualityIssue>,
}

pub fn analyze_semantic_quality(
    column: &ColumnQuality,
    semantic_type: &SemanticType,
) -> SemanticQuality {
    match semantic_type {
        SemanticType::Identifier => {
            analyze_identifier(
                column,
            )
        }

        SemanticType::Category => {
            analyze_category(
                column,
            )
        }

        // Belum ada semantic-specific penalty
        // untuk type lain.
        //
        // Mereka tetap dinilai oleh:
        // - missing
        // - consistency
        // - duplicate dataset
        _ => SemanticQuality {
            score: 100.0,
            issues: Vec::new(),
        },
    }
}

fn analyze_identifier(
    column: &ColumnQuality,
) -> SemanticQuality {
    let non_null_count =
        column
            .row_count
            .saturating_sub(
                column.missing_count,
            );

    if non_null_count <= 1 {
        return SemanticQuality {
            score: 100.0,
            issues: Vec::new(),
        };
    }

    if column.unique_rate
        >= IDENTIFIER_MIN_UNIQUE_RATE
    {
        return SemanticQuality {
            score: 100.0,
            issues: Vec::new(),
        };
    }

    let problem_rate =
        (1.0 - column.unique_rate)
            .clamp(
                0.0,
                1.0,
            );

    // Ini bukan exact duplicate row count.
    // Ini jumlah value identifier non-null
    // yang melebihi jumlah unique identifier.
    let repeated_values =
        non_null_count
            .saturating_sub(
                column.unique_count,
            );

    let issue =
        QualityIssue::new(
            QualityIssueKind::IdentifierNotUnique,

            Some(
                column.name.clone(),
            ),

            repeated_values,

            problem_rate,

            severity_from_rate(
                problem_rate,
            ),
        );

    SemanticQuality {
        score:
            score_from_rate(
                problem_rate,
            ),

        issues:
            vec![issue],
    }
}

fn analyze_category(
    column: &ColumnQuality,
) -> SemanticQuality {
    let non_null_count =
        column
            .row_count
            .saturating_sub(
                column.missing_count,
            );

    // Dataset kecil terlalu mudah memberi
    // cardinality ratio tinggi.
    if non_null_count
        < CATEGORY_MIN_ROWS
    {
        return SemanticQuality {
            score: 100.0,
            issues: Vec::new(),
        };
    }

    if column.unique_rate
        <= CATEGORY_MAX_UNIQUE_RATE
    {
        return SemanticQuality {
            score: 100.0,
            issues: Vec::new(),
        };
    }

    // Misalnya:
    //
    // max ideal = 0.50
    // actual    = 0.75
    //
    // excess ratio = 0.50
    //
    // Jadi penalty dibuat relatif terhadap
    // ruang 0.50 -> 1.00.
    let problem_rate =
        (
            (
                column.unique_rate
                    - CATEGORY_MAX_UNIQUE_RATE
            )
            / (
                1.0
                    - CATEGORY_MAX_UNIQUE_RATE
            )
        )
        .clamp(
            0.0,
            1.0,
        );

    let issue =
        QualityIssue::new(
            QualityIssueKind::CategoryCardinalityHigh,

            Some(
                column.name.clone(),
            ),

            column.unique_count,

            problem_rate,

            severity_from_rate(
                problem_rate,
            ),
        );

    SemanticQuality {
        score:
            score_from_rate(
                problem_rate,
            ),

        issues:
            vec![issue],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column_quality::analyze_column_quality;

    use polars::prelude::*;

    #[test]
    fn unique_identifier_is_healthy() {
        let series =
            Series::new(
                "customer_id".into(),
                [
                    "C001",
                    "C002",
                    "C003",
                    "C004",
                ],
            );

        let quality =
            analyze_column_quality(
                &series,
            )
            .expect(
                "Quality should succeed",
            );

        let semantic =
            analyze_semantic_quality(
                &quality,
                &SemanticType::Identifier,
            );

        assert_eq!(
            semantic.score,
            100.0
        );

        assert!(
            semantic.issues.is_empty()
        );
    }

    #[test]
    fn duplicate_identifier_creates_issue() {
        let series =
            Series::new(
                "customer_id".into(),
                [
                    "C001",
                    "C001",
                    "C002",
                    "C003",
                ],
            );

        let quality =
            analyze_column_quality(
                &series,
            )
            .expect(
                "Quality should succeed",
            );

        let semantic =
            analyze_semantic_quality(
                &quality,
                &SemanticType::Identifier,
            );

        assert!(
            semantic.score
                < 100.0
        );

        assert_eq!(
            semantic.issues.len(),
            1
        );

        assert_eq!(
            semantic.issues[0].kind,
            QualityIssueKind::IdentifierNotUnique
        );

        assert_eq!(
            semantic.issues[0].affected_rows,
            1
        );
    }

    #[test]
    fn normal_category_is_healthy() {
        let values:
            Vec<String> =
            (0..40)
                .map(
                    |index| {
                        if index % 2 == 0 {
                            "Jakarta"
                                .to_string()
                        } else {
                            "Bandung"
                                .to_string()
                        }
                    },
                )
                .collect();

        let series =
            Series::new(
                "city".into(),
                values,
            );

        let quality =
            analyze_column_quality(
                &series,
            )
            .expect(
                "Quality should succeed",
            );

        let semantic =
            analyze_semantic_quality(
                &quality,
                &SemanticType::Category,
            );

        assert_eq!(
            semantic.score,
            100.0
        );

        assert!(
            semantic.issues.is_empty()
        );
    }

    #[test]
    fn high_cardinality_category_creates_issue() {
        let values:
            Vec<String> =
            (0..20)
                .map(
                    |index| {
                        format!(
                            "Category-{index}"
                        )
                    },
                )
                .collect();

        let series =
            Series::new(
                "category".into(),
                values,
            );

        let quality =
            analyze_column_quality(
                &series,
            )
            .expect(
                "Quality should succeed",
            );

        let semantic =
            analyze_semantic_quality(
                &quality,
                &SemanticType::Category,
            );

        assert!(
            semantic.score
                < 100.0
        );

        assert_eq!(
            semantic.issues.len(),
            1
        );

        assert_eq!(
            semantic.issues[0].kind,
            QualityIssueKind::CategoryCardinalityHigh
        );
    }

    #[test]
    fn small_category_sample_is_not_penalized() {
        let series =
            Series::new(
                "city".into(),
                [
                    "Jakarta",
                    "Bandung",
                    "Surabaya",
                    "Medan",
                ],
            );

        let quality =
            analyze_column_quality(
                &series,
            )
            .expect(
                "Quality should succeed",
            );

        let semantic =
            analyze_semantic_quality(
                &quality,
                &SemanticType::Category,
            );

        assert_eq!(
            semantic.score,
            100.0
        );

        assert!(
            semantic.issues.is_empty()
        );
    }
}