use std::collections::BTreeMap;

use polars::prelude::{
    AnyValue,
    DataFrame,
    PolarsResult,
};

use serde::Serialize;
use tauri::State;

use crate::{
    error::AppError,
    state::DatasetState,
};

const MAX_LINE_POINTS: usize =
    2_000;

const MAX_SCATTER_POINTS: usize =
    5_000;

const HISTOGRAM_BINS: usize =
    12;

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct ScatterPointResponse {
    pub x: f64,
    pub y: f64,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct HistogramBinResponse {
    pub label: String,
    pub start: f64,
    pub end: f64,
    pub count: usize,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataResponse {
    pub chart_type: String,

    pub x_column: String,

    pub y_column:
        Option<String>,

    pub labels:
        Vec<String>,

    pub values:
        Vec<f64>,

    pub points:
        Vec<ScatterPointResponse>,

    pub bins:
        Vec<HistogramBinResponse>,
}

#[tauri::command]
pub fn get_chart_data(
    chart_type: String,

    x_column: String,

    y_column:
        Option<String>,

    state:
        State<'_, DatasetState>,
) -> Result<
    ChartDataResponse,
    AppError,
> {
    state
        .with_dataset(
            |dataset| {
                build_chart_data(
                    &dataset.dataframe,

                    &chart_type,

                    &x_column,

                    y_column
                        .as_deref(),
                )
                .map_err(
                    |error| {
                        error.to_string()
                    },
                )
            },
        )
        .map_err(
            map_chart_data_error,
        )
}

fn build_chart_data(
    dataframe:
        &DataFrame,

    chart_type:
        &str,

    x_column:
        &str,

    y_column:
        Option<&str>,
) -> Result<
    ChartDataResponse,
    String,
> {
    match chart_type {
        "histogram" => {
            histogram_data(
                dataframe,
                x_column,
            )
            .map_err(
                |error| {
                    error.to_string()
                },
            )
        }

        "scatter" => {
            let y_column =
                require_y_column(
                    y_column,
                )?;

            scatter_data(
                dataframe,
                x_column,
                y_column,
            )
            .map_err(
                |error| {
                    error.to_string()
                },
            )
        }

        "line" => {
            let y_column =
                require_y_column(
                    y_column,
                )?;

            line_data(
                dataframe,
                x_column,
                y_column,
            )
            .map_err(
                |error| {
                    error.to_string()
                },
            )
        }

        "bar"
        | "pie" => {
            let y_column =
                require_y_column(
                    y_column,
                )?;

            category_data(
                dataframe,
                chart_type,
                x_column,
                y_column,
            )
            .map_err(
                |error| {
                    error.to_string()
                },
            )
        }

        _ => {
            Err(
                format!(
                    "Unsupported chart type: {chart_type}",
                ),
            )
        }
    }
}

fn histogram_data(
    dataframe:
        &DataFrame,

    column_name:
        &str,
) -> PolarsResult<
    ChartDataResponse,
> {
    let column =
        dataframe.column(
            column_name,
        )?;

    let mut values =
        Vec::new();

    for row
        in 0..dataframe.height()
    {
        let value =
            column.get(
                row,
            )?;

        if let Some(
            value,
        ) =
            any_value_to_f64(
                value,
            )
        {
            if value.is_finite() {
                values.push(
                    value,
                );
            }
        }
    }

    let bins =
        build_histogram_bins(
            &values,
        );

    Ok(
        ChartDataResponse {
            chart_type:
                "histogram"
                    .to_string(),

            x_column:
                column_name
                    .to_string(),

            y_column:
                None,

            labels:
                Vec::new(),

            values:
                Vec::new(),

            points:
                Vec::new(),

            bins,
        },
    )
}

fn scatter_data(
    dataframe:
        &DataFrame,

    x_column_name:
        &str,

    y_column_name:
        &str,
) -> PolarsResult<
    ChartDataResponse,
> {
    let x_column =
        dataframe.column(
            x_column_name,
        )?;

    let y_column =
        dataframe.column(
            y_column_name,
        )?;

    let mut points =
        Vec::new();

    for row
        in 0..dataframe.height()
    {
        if points.len()
            >= MAX_SCATTER_POINTS
        {
            break;
        }

        let x =
            any_value_to_f64(
                x_column.get(
                    row,
                )?,
            );

        let y =
            any_value_to_f64(
                y_column.get(
                    row,
                )?,
            );

        if let (
            Some(x),
            Some(y),
        ) = (
            x,
            y,
        ) {
            if x.is_finite()
                && y.is_finite()
            {
                points.push(
                    ScatterPointResponse {
                        x,
                        y,
                    },
                );
            }
        }
    }

    Ok(
        ChartDataResponse {
            chart_type:
                "scatter"
                    .to_string(),

            x_column:
                x_column_name
                    .to_string(),

            y_column:
                Some(
                    y_column_name
                        .to_string(),
                ),

            labels:
                Vec::new(),

            values:
                Vec::new(),

            points,

            bins:
                Vec::new(),
        },
    )
}

fn line_data(
    dataframe:
        &DataFrame,

    x_column_name:
        &str,

    y_column_name:
        &str,
) -> PolarsResult<
    ChartDataResponse,
> {
    let x_column =
        dataframe.column(
            x_column_name,
        )?;

    let y_column =
        dataframe.column(
            y_column_name,
        )?;

    let mut labels =
        Vec::new();

    let mut values =
        Vec::new();

    for row
        in 0..dataframe.height()
    {
        if labels.len()
            >= MAX_LINE_POINTS
        {
            break;
        }

        let x_value =
            x_column.get(
                row,
            )?;

        let y_value =
            y_column.get(
                row,
            )?;

        let Some(
            y_value,
        ) =
            any_value_to_f64(
                y_value,
            )
        else {
            continue;
        };

        if !y_value.is_finite() {
            continue;
        }

        if matches!(
            x_value,
            AnyValue::Null
        ) {
            continue;
        }

        labels.push(
            x_value
                .to_string(),
        );

        values.push(
            y_value,
        );
    }

    Ok(
        ChartDataResponse {
            chart_type:
                "line"
                    .to_string(),

            x_column:
                x_column_name
                    .to_string(),

            y_column:
                Some(
                    y_column_name
                        .to_string(),
                ),

            labels,

            values,

            points:
                Vec::new(),

            bins:
                Vec::new(),
        },
    )
}

fn category_data(
    dataframe:
        &DataFrame,

    chart_type:
        &str,

    x_column_name:
        &str,

    y_column_name:
        &str,
) -> PolarsResult<
    ChartDataResponse,
> {
    let x_column =
        dataframe.column(
            x_column_name,
        )?;

    let y_column =
        dataframe.column(
            y_column_name,
        )?;

    let mut groups:
        BTreeMap<
            String,
            f64,
        > =
        BTreeMap::new();

    for row
        in 0..dataframe.height()
    {
        let category =
            x_column.get(
                row,
            )?;

        if matches!(
            category,
            AnyValue::Null
        ) {
            continue;
        }

        let value =
            y_column.get(
                row,
            )?;

        let Some(
            value,
        ) =
            any_value_to_f64(
                value,
            )
        else {
            continue;
        };

        if !value.is_finite() {
            continue;
        }

        let category =
            category
                .to_string();

        *groups
            .entry(
                category,
            )
            .or_insert(
                0.0,
            ) +=
            value;
    }

    let mut entries:
        Vec<(
            String,
            f64,
        )> =
        groups
            .into_iter()
            .collect();

    entries.sort_by(
        |left, right| {
            right
                .1
                .partial_cmp(
                    &left.1,
                )
                .unwrap_or(
                    std::cmp::Ordering::Equal,
                )
        },
    );

    let labels =
        entries
            .iter()
            .map(
                |(
                    label,
                    _,
                )| {
                    label.clone()
                },
            )
            .collect();

    let values =
        entries
            .iter()
            .map(
                |(
                    _,
                    value,
                )| {
                    *value
                },
            )
            .collect();

    Ok(
        ChartDataResponse {
            chart_type:
                chart_type
                    .to_string(),

            x_column:
                x_column_name
                    .to_string(),

            y_column:
                Some(
                    y_column_name
                        .to_string(),
                ),

            labels,

            values,

            points:
                Vec::new(),

            bins:
                Vec::new(),
        },
    )
}

fn build_histogram_bins(
    values:
        &[f64],
) -> Vec<
    HistogramBinResponse,
> {
    if values.is_empty() {
        return Vec::new();
    }

    let min =
        values
            .iter()
            .copied()
            .fold(
                f64::INFINITY,
                f64::min,
            );

    let max =
        values
            .iter()
            .copied()
            .fold(
                f64::NEG_INFINITY,
                f64::max,
            );

    if min == max {
        return vec![
            HistogramBinResponse {
                label:
                    format_number(
                        min,
                    ),

                start:
                    min,

                end:
                    max,

                count:
                    values.len(),
            },
        ];
    }

    let width =
        (
            max - min
        )
        / HISTOGRAM_BINS
            as f64;

    let mut counts =
        vec![
            0_usize;
            HISTOGRAM_BINS
        ];

    for value in values {
        let mut index =
            (
                (
                    *value - min
                )
                / width
            )
            .floor()
                as usize;

        if index
            >= HISTOGRAM_BINS
        {
            index =
                HISTOGRAM_BINS
                    - 1;
        }

        counts[
            index
        ] +=
            1;
    }

    counts
        .into_iter()
        .enumerate()
        .map(
            |(
                index,
                count,
            )| {
                let start =
                    min
                    + width
                        * index
                            as f64;

                let end =
                    if index
                        == HISTOGRAM_BINS - 1
                    {
                        max
                    } else {
                        start
                        + width
                    };

                HistogramBinResponse {
                    label:
                        format!(
                            "{} – {}",
                            format_number(
                                start,
                            ),
                            format_number(
                                end,
                            ),
                        ),

                    start,

                    end,

                    count,
                }
            },
        )
        .collect()
}

fn any_value_to_f64(
    value:
        AnyValue<'_>,
) -> Option<f64> {
    match value {
        AnyValue::Int8(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::Int16(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::Int32(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::Int64(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::UInt8(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::UInt16(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::UInt32(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::UInt64(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::Float32(
            value,
        ) => {
            Some(
                value as f64,
            )
        }

        AnyValue::Float64(
            value,
        ) => {
            Some(
                value,
            )
        }

        _ => None,
    }
}

fn require_y_column(
    value:
        Option<&str>,
) -> Result<
    &str,
    String,
> {
    value.ok_or_else(
        || {
            "This chart requires a Y column"
                .to_string()
        },
    )
}

fn format_number(
    value:
        f64,
) -> String {
    format!(
        "{value:.2}"
    )
}

fn map_chart_data_error(
    error:
        String,
) -> AppError {
    if error
        == "No dataset is currently loaded"
    {
        AppError::DatasetNotLoaded
    } else {
        AppError::Charts(
            error,
        )
    }
}