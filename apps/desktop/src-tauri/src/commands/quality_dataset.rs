use serde::Serialize;
use tauri::State;

use flowtren_core::semantic_type::SemanticType;

use flowtren_quality::{
    column_quality::ColumnQuality,
    comparison::{
        compare_quality,
        QualityComparison,
    },
    dataset_quality::{
        analyze_dataset_quality,
        DatasetQuality,
    },
    issue::{
        QualityIssue,
        QualityIssueKind,
    },
    severity::QualitySeverity,
};

use crate::{
    error::AppError,
    state::DatasetState,
};

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct QualityIssueResponse {
    pub kind:
        String,

    pub column:
        Option<String>,

    pub affected_rows:
        usize,

    pub rate:
        f64,

    pub severity:
        String,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct ColumnQualityResponse {
    pub name:
        String,

    pub semantic_type:
        Option<String>,

    pub row_count:
        usize,

    pub missing_count:
        usize,

    pub missing_rate:
        f64,

    pub missing_severity:
        String,

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

    pub score:
        f64,

    pub issues:
        Vec<
            QualityIssueResponse,
        >,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct DatasetQualityResponse {
    pub row_count:
        usize,

    pub column_count:
        usize,

    pub duplicate_count:
        usize,

    pub duplicate_rate:
        f64,

    pub duplicate_severity:
        String,

    pub score:
        f64,

    pub issues:
        Vec<
            QualityIssueResponse,
        >,

    pub columns:
        Vec<
            ColumnQualityResponse,
        >,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct QualityComparisonResponse {
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

#[tauri::command]
pub fn analyze_quality(
    state:
        State<'_, DatasetState>,
) -> Result<
    DatasetQualityResponse,
    AppError,
> {
    let dataframe =
        state
            .dataframe_snapshot()
            .map_err(
                map_quality_error,
            )?;

    let quality =
        analyze_dataset_quality(
            &dataframe,
        )
        .map_err(
            |error| {
                AppError::Quality(
                    error
                        .to_string(),
                )
            },
        )?;

    Ok(
        dataset_quality_response(
            &quality,
        ),
    )
}

#[tauri::command]
pub fn compare_dataset_quality(
    state:
        State<'_, DatasetState>,
) -> Result<
    QualityComparisonResponse,
    AppError,
> {
    let (
        original_dataframe,
        dataframe,
    ) =
        state
            .quality_comparison_snapshot()
            .map_err(
                map_quality_error,
            )?;

    let before =
        analyze_dataset_quality(
            &original_dataframe,
        )
        .map_err(
            |error| {
                AppError::Quality(
                    error
                        .to_string(),
                )
            },
        )?;

    let after =
        analyze_dataset_quality(
            &dataframe,
        )
        .map_err(
            |error| {
                AppError::Quality(
                    error
                        .to_string(),
                )
            },
        )?;

    let comparison =
        compare_quality(
            &before,
            &after,
        );

    Ok(
        quality_comparison_response(
            &comparison,
        ),
    )
}

fn dataset_quality_response(
    quality:
        &DatasetQuality,
) -> DatasetQualityResponse {
    DatasetQualityResponse {
        row_count:
            quality
                .row_count,

        column_count:
            quality
                .column_count,

        duplicate_count:
            quality
                .duplicate_count,

        duplicate_rate:
            quality
                .duplicate_rate,

        duplicate_severity:
            severity_name(
                &quality
                    .duplicate_severity,
            )
            .to_string(),

        score:
            quality
                .score,

        issues:
            quality
                .issues
                .iter()
                .map(
                    quality_issue_response,
                )
                .collect(),

        columns:
            quality
                .columns
                .iter()
                .map(
                    column_quality_response,
                )
                .collect(),
    }
}

fn column_quality_response(
    quality:
        &ColumnQuality,
) -> ColumnQualityResponse {
    ColumnQualityResponse {
        name:
            quality
                .name
                .clone(),

        semantic_type:
            quality
                .semantic_type
                .as_ref()
                .map(
                    |semantic_type| {
                        semantic_type_name(
                            semantic_type,
                        )
                        .to_string()
                    },
                ),

        row_count:
            quality
                .row_count,

        missing_count:
            quality
                .missing_count,

        missing_rate:
            quality
                .missing_rate,

        missing_severity:
            severity_name(
                &quality
                    .missing_severity,
            )
            .to_string(),

        unique_count:
            quality
                .unique_count,

        unique_rate:
            quality
                .unique_rate,

        inconsistent_count:
            quality
                .inconsistent_count,

        inconsistency_rate:
            quality
                .inconsistency_rate,

        inconsistent_groups:
            quality
                .inconsistent_groups,

        score:
            quality
                .score,

        issues:
            quality
                .issues
                .iter()
                .map(
                    quality_issue_response,
                )
                .collect(),
    }
}

fn quality_issue_response(
    issue:
        &QualityIssue,
) -> QualityIssueResponse {
    QualityIssueResponse {
        kind:
            issue_kind_name(
                &issue.kind,
            )
            .to_string(),

        column:
            issue
                .column
                .clone(),

        affected_rows:
            issue
                .affected_rows,

        rate:
            issue
                .rate,

        severity:
            severity_name(
                &issue
                    .severity,
            )
            .to_string(),
    }
}

fn quality_comparison_response(
    comparison:
        &QualityComparison,
) -> QualityComparisonResponse {
    QualityComparisonResponse {
        before_score:
            comparison
                .before_score,

        after_score:
            comparison
                .after_score,

        score_change:
            comparison
                .score_change,

        improved:
            comparison
                .improved,

        before_issue_count:
            comparison
                .before_issue_count,

        after_issue_count:
            comparison
                .after_issue_count,

        resolved_issue_count:
            comparison
                .resolved_issue_count,

        new_issue_count:
            comparison
                .new_issue_count,

        remaining_issue_count:
            comparison
                .remaining_issue_count,
    }
}

fn severity_name(
    severity:
        &QualitySeverity,
) -> &'static str {
    match severity {
        QualitySeverity::Healthy => {
            "healthy"
        }

        QualitySeverity::Low => {
            "low"
        }

        QualitySeverity::Medium => {
            "medium"
        }

        QualitySeverity::High => {
            "high"
        }
    }
}

fn issue_kind_name(
    kind:
        &QualityIssueKind,
) -> &'static str {
    match kind {
        QualityIssueKind::MissingValues => {
            "missingValues"
        }

        QualityIssueKind::DuplicateRows => {
            "duplicateRows"
        }

        QualityIssueKind::InconsistentValues => {
            "inconsistentValues"
        }

        QualityIssueKind::IdentifierNotUnique => {
            "identifierNotUnique"
        }

        QualityIssueKind::CategoryCardinalityHigh => {
            "categoryCardinalityHigh"
        }
    }
}

fn semantic_type_name(
    semantic_type:
        &SemanticType,
) -> &'static str {
    match semantic_type {
        SemanticType::Integer => {
            "integer"
        }

        SemanticType::Decimal => {
            "decimal"
        }

        SemanticType::Currency => {
            "currency"
        }

        SemanticType::Percentage => {
            "percentage"
        }

        SemanticType::Category => {
            "category"
        }

        SemanticType::Boolean => {
            "boolean"
        }

        SemanticType::Date => {
            "date"
        }

        SemanticType::DateTime => {
            "datetime"
        }

        SemanticType::Email => {
            "email"
        }

        SemanticType::Phone => {
            "phone"
        }

        SemanticType::Identifier => {
            "identifier"
        }

        SemanticType::Text => {
            "text"
        }

        SemanticType::Unknown => {
            "unknown"
        }
    }
}

fn map_quality_error(
    error:
        String,
) -> AppError {
    if error
        == "No dataset is currently loaded"
    {
        AppError::DatasetNotLoaded
    } else {
        AppError::Quality(
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_severity_names_are_stable() {
        assert_eq!(
            severity_name(
                &QualitySeverity::Healthy,
            ),
            "healthy"
        );

        assert_eq!(
            severity_name(
                &QualitySeverity::High,
            ),
            "high"
        );
    }

    #[test]
    fn quality_issue_names_are_stable() {
        assert_eq!(
            issue_kind_name(
                &QualityIssueKind::MissingValues,
            ),
            "missingValues"
        );

        assert_eq!(
            issue_kind_name(
                &QualityIssueKind::DuplicateRows,
            ),
            "duplicateRows"
        );
    }

    #[test]
    fn semantic_names_are_stable() {
        assert_eq!(
            semantic_type_name(
                &SemanticType::Currency,
            ),
            "currency"
        );

        assert_eq!(
            semantic_type_name(
                &SemanticType::Identifier,
            ),
            "identifier"
        );
    }
}