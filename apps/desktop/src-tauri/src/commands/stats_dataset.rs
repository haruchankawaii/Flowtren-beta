use polars::prelude::*;
use serde::Serialize;
use tauri::State;

use flowtren_stats::{
    analyze_descriptive,
    analyze_index_trend,
    analyze_percentiles,
    DescriptiveStats,
    PercentileStats,
    TrendDirection,
    TrendStats,
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
pub struct DescriptiveStatsResponse {
    pub row_count: usize,

    pub count: usize,

    pub null_count: usize,

    pub sum: Option<f64>,

    pub mean: Option<f64>,

    pub min: Option<f64>,

    pub max: Option<f64>,

    pub std_dev: Option<f64>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct PercentileStatsResponse {
    pub p25: Option<f64>,

    pub median: Option<f64>,

    pub p75: Option<f64>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct TrendStatsResponse {
    pub pair_count: usize,

    pub slope: Option<f64>,

    pub intercept: Option<f64>,

    pub r_squared: Option<f64>,

    pub direction: Option<String>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct NumericColumnStatsResponse {
    pub column: String,

    pub descriptive:
        DescriptiveStatsResponse,

    pub percentiles:
        PercentileStatsResponse,

    pub trend:
        TrendStatsResponse,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct CorrelationResponse {
    pub left_column: String,

    pub right_column: String,

    pub pair_count: usize,

    pub pearson: Option<f64>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsResponse {
    pub row_count: usize,

    pub column_count: usize,

    pub numeric_column_count: usize,

    pub columns:
        Vec<NumericColumnStatsResponse>,

    pub correlations:
        Vec<CorrelationResponse>,
}

#[tauri::command]
pub fn analyze_statistics(
    state: State<'_, DatasetState>,
) -> Result<
    StatisticsResponse,
    AppError,
> {
    state
        .with_dataset(
            |dataset| {
                analyze_dataframe_statistics(
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
            map_statistics_error,
        )
}

fn analyze_dataframe_statistics(
    df: &DataFrame,
) -> PolarsResult<StatisticsResponse> {
    let mut columns:
        Vec<NumericColumnStatsResponse> =
        Vec::new();

    let mut numeric_column_count =
        0_usize;

    for column
        in df.columns()
            .iter()
    {
        let series =
            column
                .as_materialized_series();

        if !is_numeric_series(
            series,
        ) {
            continue;
        }

        numeric_column_count +=
            1;

        let descriptive =
            analyze_descriptive(
                series,
            )?;

        let percentiles =
            analyze_percentiles(
                series,
            )?;

        let trend =
            analyze_index_trend(
                series,
            )?;

        columns.push(
            NumericColumnStatsResponse {
                column:
                    series
                        .name()
                        .to_string(),

                descriptive:
                    descriptive_response(
                        &descriptive,
                    ),

                percentiles:
                    percentile_response(
                        &percentiles,
                    ),

                trend:
                    trend_response(
                        &trend,
                    ),
            },
        );
    }

    Ok(
        StatisticsResponse {
            row_count:
                df.height(),

            column_count:
                df.width(),

            numeric_column_count,

            columns,

            correlations:
                Vec::new(),
        },
    )
}

fn descriptive_response(
    stats: &DescriptiveStats,
) -> DescriptiveStatsResponse {
    DescriptiveStatsResponse {
        row_count:
            stats.row_count,

        count:
            stats.count,

        null_count:
            stats.null_count,

        sum:
            stats.sum,

        mean:
            stats.mean,

        min:
            stats.min,

        max:
            stats.max,

        std_dev:
            stats.std_dev,
    }
}

fn percentile_response(
    stats: &PercentileStats,
) -> PercentileStatsResponse {
    PercentileStatsResponse {
        p25:
            stats.p25,

        median:
            stats.median,

        p75:
            stats.p75,
    }
}

fn trend_response(
    stats: &TrendStats,
) -> TrendStatsResponse {
    TrendStatsResponse {
        pair_count:
            stats.pair_count,

        slope:
            stats.slope,

        intercept:
            stats.intercept,

        r_squared:
            stats.r_squared,

        direction:
            stats
                .direction
                .as_ref()
                .map(
                    |direction| {
                        trend_direction_name(
                            direction,
                        )
                        .to_string()
                    },
                ),
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

fn trend_direction_name(
    direction: &TrendDirection,
) -> &'static str {
    match direction {
        TrendDirection::Increasing => {
            "increasing"
        }

        TrendDirection::Decreasing => {
            "decreasing"
        }

        TrendDirection::Flat => {
            "flat"
        }
    }
}

fn map_statistics_error(
    error: String,
) -> AppError {
    if error
        == "No dataset is currently loaded"
    {
        AppError::DatasetNotLoaded
    } else {
        AppError::Statistics(
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_dtype_is_detected() {
        let series =
            Series::new(
                "value".into(),
                [
                    1.0_f64,
                    2.0,
                    3.0,
                ],
            );

        assert!(
            is_numeric_series(
                &series,
            )
        );
    }

    #[test]
    fn string_dtype_is_not_numeric() {
        let series =
            Series::new(
                "name".into(),
                [
                    "Alice",
                    "Bob",
                ],
            );

        assert!(
            !is_numeric_series(
                &series,
            )
        );
    }

    #[test]
    fn trend_names_are_stable() {
        assert_eq!(
            trend_direction_name(
                &TrendDirection::Increasing,
            ),
            "increasing"
        );

        assert_eq!(
            trend_direction_name(
                &TrendDirection::Decreasing,
            ),
            "decreasing"
        );

        assert_eq!(
            trend_direction_name(
                &TrendDirection::Flat,
            ),
            "flat"
        );
    }

    #[test]
    fn trend_response_supports_missing_direction() {
        let stats =
            TrendStats {
                pair_count: 0,

                slope: None,

                intercept: None,

                r_squared: None,

                direction: None,
            };

        let response =
            trend_response(
                &stats,
            );

        assert_eq!(
            response.direction,
            None
        );
    }

    #[test]
    fn statistics_skip_correlations_for_fast_explore() {
        let df =
            DataFrame::new(
                3,
                vec![
                    Series::new(
                        "x".into(),
                        [
                            1.0_f64,
                            2.0,
                            3.0,
                        ],
                    )
                    .into(),

                    Series::new(
                        "y".into(),
                        [
                            3.0_f64,
                            2.0,
                            1.0,
                        ],
                    )
                    .into(),
                ],
            )
            .unwrap();

        let result =
            analyze_dataframe_statistics(
                &df,
            )
            .unwrap();

        assert_eq!(
            result.numeric_column_count,
            2
        );

        assert!(
            result.correlations
                .is_empty()
        );
    }
}