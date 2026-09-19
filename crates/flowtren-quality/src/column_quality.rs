use flowtren_core::semantic_type::SemanticType;
use polars::prelude::*;

use crate::{
    cardinality::{
        CardinalityMetrics,
        analyze_cardinality,
    },
    consistency::{
        ConsistencyMetrics,
        analyze_consistency,
    },
    issue::{
        QualityIssue,
        QualityIssueKind,
    },
    missing::{
        MissingMetrics,
        analyze_missing,
    },
    score::{
        score_from_inconsistency_rate,
        score_from_missing_rate,
        weighted_score,
    },
    severity::{
        QualitySeverity,
        severity_from_rate,
    },
};

#[derive(Debug, Clone)]
pub struct ColumnQuality {
    pub name: String,

    pub semantic_type:
        Option<SemanticType>,

    pub row_count:
        usize,

    pub missing_count:
        usize,

    pub missing_rate:
        f64,

    pub missing_severity:
        QualitySeverity,

    pub unique_count:
        usize,

    pub unique_rate:
        f64,

    pub inconsistent_count:
        usize,

    pub inconsistency_rate:
        f64,

    pub inconsistent_groups:
        usize,

    pub issues:
        Vec<QualityIssue>,

    pub score:
        f64,
}

pub fn analyze_column_quality(
    series: &Series,
) -> PolarsResult<ColumnQuality> {
    let MissingMetrics {
        missing_count,
        missing_rate,
    } =
        analyze_missing(
            series,
        );

    let CardinalityMetrics {
        unique_count,
        unique_rate,
    } =
        analyze_cardinality(
            series,
        )?;

    let ConsistencyMetrics {
        inconsistent_count,
        inconsistency_rate,
        inconsistent_groups,
    } =
        analyze_consistency(
            series,
        );

    let missing_score =
        score_from_missing_rate(
            missing_rate,
        );

    let consistency_score =
        score_from_inconsistency_rate(
            inconsistency_rate,
        );

    let score =
        weighted_score(
            &[
                (
                    missing_score,
                    0.60,
                ),
                (
                    consistency_score,
                    0.40,
                ),
            ],
        );

    let mut issues =
        Vec::new();

    if missing_count > 0 {
        issues.push(
            QualityIssue::new(
                QualityIssueKind::MissingValues,

                Some(
                    series
                        .name()
                        .to_string(),
                ),

                missing_count,

                missing_rate,

                severity_from_rate(
                    missing_rate,
                ),
            ),
        );
    }

    if inconsistent_count > 0 {
        issues.push(
            QualityIssue::new(
                QualityIssueKind::InconsistentValues,

                Some(
                    series
                        .name()
                        .to_string(),
                ),

                inconsistent_count,

                inconsistency_rate,

                severity_from_rate(
                    inconsistency_rate,
                ),
            ),
        );
    }

    Ok(
        ColumnQuality {
            name:
                series
                    .name()
                    .to_string(),

            semantic_type:
                None,

            row_count:
                series.len(),

            missing_count,
            missing_rate,

            missing_severity:
                severity_from_rate(
                    missing_rate,
                ),

            unique_count,
            unique_rate,

            inconsistent_count,
            inconsistency_rate,
            inconsistent_groups,

            issues,

            score,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_column_quality() {
        let series =
            Series::new(
                "city".into(),
                [
                    Some("Jakarta"),
                    Some("Jakarta"),
                    Some("jakarta"),
                    None,
                ],
            );

        let quality =
            analyze_column_quality(
                &series,
            )
            .expect(
                "Column quality should succeed",
            );

        assert_eq!(
            quality.name,
            "city"
        );

        assert_eq!(
            quality.row_count,
            4
        );

        assert_eq!(
            quality.missing_count,
            1
        );

        assert_eq!(
            quality.missing_rate,
            0.25
        );

        assert_eq!(
            quality.unique_count,
            2
        );

        assert_eq!(
            quality.inconsistent_count,
            1
        );

        assert_eq!(
            quality.inconsistent_groups,
            1
        );

        assert_eq!(
            quality.issues.len(),
            2
        );

        assert!(
            quality.semantic_type
                .is_none()
        );

        assert!(
            quality.score
                < 100.0
        );
    }

    #[test]
    fn perfect_column_scores_100() {
        let series =
            Series::new(
                "city".into(),
                [
                    "Jakarta",
                    "Bandung",
                    "Surabaya",
                ],
            );

        let quality =
            analyze_column_quality(
                &series,
            )
            .expect(
                "Column quality should succeed",
            );

        assert_eq!(
            quality.score,
            100.0
        );

        assert!(
            quality.issues.is_empty()
        );
    }
}