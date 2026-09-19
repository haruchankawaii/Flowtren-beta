use std::collections::HashMap;

use polars::prelude::*;
use serde::Serialize;
use tauri::State;

use flowtren_insights::{
    Insight,
    InsightConfidence,
    InsightEngine,
    InsightEvidence,
    InsightKind,
    InsightSeverity,
};

use flowtren_quality::{
    analyze_dataset_quality,
};

use flowtren_stats::{
    analyze_correlation,
    analyze_descriptive,
    analyze_index_trend,
    analyze_outliers,
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
#[serde(rename_all = "camelCase")]
pub struct InsightEvidenceResponse {
    pub label: String,

    pub value: String,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct InsightResponse {
    pub kind: String,

    pub title: String,

    pub summary: String,

    pub columns:
        Vec<String>,

    pub severity:
        String,

    pub confidence:
        String,

    pub priority_score:
        f64,

    pub evidence:
        Vec<InsightEvidenceResponse>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct InsightsResponse {
    pub insight_count:
        usize,

    pub insights:
        Vec<InsightResponse>,
}

#[tauri::command]
pub fn generate_insights(
    state: State<'_, DatasetState>,
) -> Result<
    InsightsResponse,
    AppError,
> {
    state
        .with_dataset(
            |dataset| {
                generate_dataframe_insights(
                    &dataset.dataframe,
                )
                .map_err(
                    |error| {
                        error.to_string()
                    },
                )
            },
        )
        .map_err(
            map_insight_error,
        )
}

fn generate_dataframe_insights(
    df: &DataFrame,
) -> PolarsResult<InsightsResponse> {
    let mut engine =
        InsightEngine::new();

    add_quality_insights(
        df,
        &mut engine,
    )?;

    add_numeric_insights(
        df,
        &mut engine,
    )?;

    add_correlation_insights(
        df,
        &mut engine,
    )?;

    add_category_insights(
        df,
        &mut engine,
    )?;

    let insights =
        engine.finish();

    let responses:
        Vec<InsightResponse> =
        insights
            .iter()
            .map(
                insight_response,
            )
            .collect();

    Ok(
        InsightsResponse {
            insight_count:
                responses.len(),

            insights:
                responses,
        },
    )
}

fn add_quality_insights(
    df: &DataFrame,

    engine:
        &mut InsightEngine,
) -> PolarsResult<()> {
    let quality =
        analyze_dataset_quality(
            df,
        )?;

    engine
        .add_duplicate_analysis(
            &quality,
        );

    for column
        in &quality.columns
    {
        engine
            .add_missing_analysis(
                column,
            );
    }

    Ok(())
}

fn add_numeric_insights(
    df: &DataFrame,

    engine:
        &mut InsightEngine,
) -> PolarsResult<()> {
    for column
        in df.columns()
    {
        let series =
            column
                .as_materialized_series();

        if !is_numeric_series(
            series,
        ) {
            continue;
        }

        let column_name =
            series.name()
                .to_string();

        let descriptive =
            analyze_descriptive(
                series,
            )?;

        engine
            .add_volatility_analysis(
                &column_name,

                &descriptive,
            );

        let outliers =
            analyze_outliers(
                series,
            )?;

        engine
            .add_outlier_analysis(
                &column_name,

                &outliers,
            );

        let trend =
            analyze_index_trend(
                series,
            )?;

        engine
            .add_trend_analysis(
                &column_name,

                &trend,
            );

        if let Some((
            first,
            last,
            observation_count,
        )) =
            first_last_numeric(
                series,
            )?
        {
            engine
                .add_growth_analysis(
                    &column_name,

                    first,

                    last,

                    observation_count,
                );

            engine
                .add_decline_analysis(
                    &column_name,

                    first,

                    last,

                    observation_count,
                );
        }
    }

    Ok(())
}

fn add_correlation_insights(
    df: &DataFrame,

    engine:
        &mut InsightEngine,
) -> PolarsResult<()> {
    let numeric_indices:
        Vec<usize> =
        df.columns()
            .iter()
            .enumerate()
            .filter_map(
                |(
                    index,
                    column,
                )| {
                    let series =
                        column
                            .as_materialized_series();

                    if is_numeric_series(
                        series,
                    ) {
                        Some(
                            index,
                        )
                    } else {
                        None
                    }
                },
            )
            .collect();

    for left_position
        in 0..numeric_indices.len()
    {
        for right_position
            in (left_position + 1)
                ..numeric_indices.len()
        {
            let left =
                df.columns()[
                    numeric_indices[
                        left_position
                    ]
                ]
                .as_materialized_series();

            let right =
                df.columns()[
                    numeric_indices[
                        right_position
                    ]
                ]
                .as_materialized_series();

            let stats =
                analyze_correlation(
                    left,
                    right,
                )?;

            engine
                .add_correlation_analysis(
                    left.name()
                        .as_str(),

                    right.name()
                        .as_str(),

                    &stats,
                );
        }
    }

    Ok(())
}

fn add_category_insights(
    df: &DataFrame,

    engine:
        &mut InsightEngine,
) -> PolarsResult<()> {
    for column
        in df.columns()
    {
        let series =
            column
                .as_materialized_series();

        if series.dtype()
            != &DataType::String
        {
            continue;
        }

        let strings =
            series.str()?;

        let mut counts:
            HashMap<String, usize> =
            HashMap::new();

        let mut total_count =
            0_usize;

        for value
            in strings.iter()
                .flatten()
        {
            let value =
                value.trim();

            if value.is_empty() {
                continue;
            }

            total_count += 1;

            *counts
                .entry(
                    value.to_string(),
                )
                .or_insert(0) += 1;
        }

        if total_count == 0
            || counts.is_empty()
        {
            continue;
        }

        let dominant =
            counts
                .iter()
                .max_by_key(
                    |(_, count)| {
                        *count
                    },
                );

        if let Some((
            dominant_value,
            dominant_count,
        )) = dominant
        {
            engine
                .add_dominance_analysis(
                    series.name()
                        .as_str(),

                    dominant_value,

                    *dominant_count,

                    total_count,
                );
        }

        let category_counts:
            Vec<usize> =
            counts
                .values()
                .copied()
                .collect();

        engine
            .add_concentration_analysis(
                series.name()
                    .as_str(),

                &category_counts,
            );
    }

    Ok(())
}

fn first_last_numeric(
    series: &Series,
) -> PolarsResult<
    Option<(
        f64,
        f64,
        usize,
    )>,
> {
    let casted =
        series.cast(
            &DataType::Float64,
        )?;

    let values =
        casted.f64()?;

    let valid_values:
        Vec<f64> =
        values
            .iter()
            .flatten()
            .filter(
                |value| {
                    value.is_finite()
                },
            )
            .collect();

    if valid_values.len()
        < 2
    {
        return Ok(None);
    }

    let first =
        valid_values[0];

    let last =
        valid_values[
            valid_values.len()
                - 1
        ];

    Ok(
        Some((
            first,
            last,
            valid_values.len(),
        )),
    )
}

fn insight_response(
    insight: &Insight,
) -> InsightResponse {
    InsightResponse {
        kind:
            insight_kind_name(
                &insight.kind,
            )
            .to_string(),

        title:
            insight.title
                .clone(),

        summary:
            insight.summary
                .clone(),

        columns:
            insight.columns
                .clone(),

        severity:
            insight_severity_name(
                &insight.severity,
            )
            .to_string(),

        confidence:
            insight_confidence_name(
                &insight.confidence,
            )
            .to_string(),

        priority_score:
            insight.priority_score,

        evidence:
            insight.evidence
                .iter()
                .map(
                    evidence_response,
                )
                .collect(),
    }
}

fn evidence_response(
    evidence:
        &InsightEvidence,
) -> InsightEvidenceResponse {
    InsightEvidenceResponse {
        label:
            evidence.label
                .clone(),

        value:
            evidence.value
                .clone(),
    }
}

fn insight_kind_name(
    kind: &InsightKind,
) -> &'static str {
    match kind {
        InsightKind::Growth => {
            "growth"
        }

        InsightKind::Decline => {
            "decline"
        }

        InsightKind::Dominance => {
            "dominance"
        }

        InsightKind::Outlier => {
            "outlier"
        }

        InsightKind::MissingValues => {
            "missingValues"
        }

        InsightKind::DuplicateRows => {
            "duplicateRows"
        }

        InsightKind::Correlation => {
            "correlation"
        }

        InsightKind::Volatility => {
            "volatility"
        }

        InsightKind::Concentration => {
            "concentration"
        }

        InsightKind::Trend => {
            "trend"
        }
    }
}

fn insight_severity_name(
    severity:
        &InsightSeverity,
) -> &'static str {
    match severity {
        InsightSeverity::Info => {
            "info"
        }

        InsightSeverity::Low => {
            "low"
        }

        InsightSeverity::Medium => {
            "medium"
        }

        InsightSeverity::High => {
            "high"
        }
    }
}

fn insight_confidence_name(
    confidence:
        &InsightConfidence,
) -> &'static str {
    match confidence {
        InsightConfidence::Low => {
            "low"
        }

        InsightConfidence::Medium => {
            "medium"
        }

        InsightConfidence::High => {
            "high"
        }
    }
}

fn is_numeric_series(
    series: &Series,
) -> bool {
    matches!(
        series.dtype(),

        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float32
            | DataType::Float64
    )
}

fn map_insight_error(
    error: String,
) -> AppError {
    if error
        == "No dataset is currently loaded"
    {
        AppError::DatasetNotLoaded
    } else {
        AppError::Insights(
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insight_kind_names_are_stable() {
        assert_eq!(
            insight_kind_name(
                &InsightKind::Growth,
            ),
            "growth"
        );

        assert_eq!(
            insight_kind_name(
                &InsightKind::Correlation,
            ),
            "correlation"
        );

        assert_eq!(
            insight_kind_name(
                &InsightKind::MissingValues,
            ),
            "missingValues"
        );
    }

    #[test]
    fn severity_names_are_stable() {
        assert_eq!(
            insight_severity_name(
                &InsightSeverity::High,
            ),
            "high"
        );
    }

    #[test]
    fn confidence_names_are_stable() {
        assert_eq!(
            insight_confidence_name(
                &InsightConfidence::High,
            ),
            "high"
        );
    }

    #[test]
    fn extracts_first_and_last_numeric_values() {
        let series =
            Series::new(
                "value".into(),
                [
                    Some(10.0_f64),
                    None,
                    Some(20.0),
                    Some(30.0),
                ],
            );

        let result =
            first_last_numeric(
                &series,
            )
            .unwrap()
            .unwrap();

        assert_eq!(
            result.0,
            10.0
        );

        assert_eq!(
            result.1,
            30.0
        );

        assert_eq!(
            result.2,
            3
        );
    }
}