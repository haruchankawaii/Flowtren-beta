use serde::Serialize;
use tauri::State;

use flowtren_charts::{
    chart_is_compatible,
    recommend_single_column,
    recommend_two_columns,
    ChartConfidence,
    ChartRecommendation,
    ChartType,
};

use flowtren_core::semantic_type::SemanticType;

use flowtren_profiler::dataset_profile::profile_dataframe;

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
pub struct ChartRecommendationResponse {
    pub chart_type: String,

    pub x_column: String,

    pub y_column: Option<String>,

    pub confidence: String,

    pub reason: String,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct ChartRecommendationsResponse {
    pub recommendation_count: usize,

    pub recommendations:
        Vec<ChartRecommendationResponse>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct CompatibleChartsResponse {
    pub x_column: String,

    pub y_column: Option<String>,

    pub chart_types: Vec<String>,
}

#[tauri::command]
pub fn recommend_charts(
    state: State<'_, DatasetState>,
) -> Result<
    ChartRecommendationsResponse,
    AppError,
> {
    state
        .with_dataset(
            |dataset| {
                let profile =
                    profile_dataframe(
                        &dataset.dataframe,
                    )
                    .map_err(
                        |error| {
                            error.to_string()
                        },
                    )?;

                let mut recommendations:
                    Vec<ChartRecommendation> =
                    Vec::new();

                // -------------------------------------
                // Single-column recommendations
                // -------------------------------------

                for column
                    in &profile.columns
                {
                    if let Some(
                        recommendation,
                    ) =
                        recommend_single_column(
                            &column.name,
                            &column.semantic_type,
                        )
                    {
                        recommendations.push(
                            recommendation,
                        );
                    }
                }

                // -------------------------------------
                // Two-column recommendations
                // -------------------------------------

                for left_index
                    in 0..profile.columns.len()
                {
                    for right_index
                        in (left_index + 1)
                            ..profile.columns.len()
                    {
                        let left =
                            &profile.columns[
                                left_index
                            ];

                        let right =
                            &profile.columns[
                                right_index
                            ];

                        // Coba orientasi:
                        //
                        // left -> x
                        // right -> y
                        if let Some(
                            recommendation,
                        ) =
                            recommend_two_columns(
                                &left.name,

                                &left.semantic_type,

                                &right.name,

                                &right.semantic_type,

                                Some(
                                    left.unique_count,
                                ),
                            )
                        {
                            recommendations.push(
                                recommendation,
                            );

                            continue;
                        }

                        // Jika tidak compatible,
                        // coba orientasi terbalik.
                        //
                        // Ini penting untuk kasus:
                        //
                        // numeric + category
                        //
                        // karena chart engine
                        // mengharapkan category
                        // sebagai x dan numeric
                        // sebagai y.
                        if let Some(
                            recommendation,
                        ) =
                            recommend_two_columns(
                                &right.name,

                                &right.semantic_type,

                                &left.name,

                                &left.semantic_type,

                                Some(
                                    right.unique_count,
                                ),
                            )
                        {
                            recommendations.push(
                                recommendation,
                            );
                        }
                    }
                }

                let responses:
                    Vec<
                        ChartRecommendationResponse,
                    > =
                    recommendations
                        .iter()
                        .map(
                            recommendation_response,
                        )
                        .collect();

                Ok(
                    ChartRecommendationsResponse {
                        recommendation_count:
                            responses.len(),

                        recommendations:
                            responses,
                    },
                )
            },
        )
        .map_err(
            map_chart_error,
        )
}

#[tauri::command]
pub fn get_compatible_charts(
    x_column: String,

    y_column: Option<String>,

    state: State<'_, DatasetState>,
) -> Result<
    CompatibleChartsResponse,
    AppError,
> {
    state
        .with_dataset(
            |dataset| {
                let profile =
                    profile_dataframe(
                        &dataset.dataframe,
                    )
                    .map_err(
                        |error| {
                            error.to_string()
                        },
                    )?;

                let x_profile =
                    profile
                        .columns
                        .iter()
                        .find(
                            |column| {
                                column.name
                                    == x_column
                            },
                        )
                        .ok_or_else(
                            || {
                                format!(
                                    "Column '{}' does not exist",
                                    x_column
                                )
                            },
                        )?;

                let y_profile =
                    match y_column
                        .as_ref()
                    {
                        Some(
                            y_column_name,
                        ) => {
                            Some(
                                profile
                                    .columns
                                    .iter()
                                    .find(
                                        |column| {
                                            &column.name
                                                == y_column_name
                                        },
                                    )
                                    .ok_or_else(
                                        || {
                                            format!(
                                                "Column '{}' does not exist",
                                                y_column_name
                                            )
                                        },
                                    )?,
                            )
                        }

                        None => None,
                    };

                let chart_types =
                    compatible_chart_types(
                        &x_profile
                            .semantic_type,

                        y_profile.map(
                            |profile| {
                                &profile
                                    .semantic_type
                            },
                        ),

                        Some(
                            x_profile
                                .unique_count,
                        ),
                    );

                Ok(
                    CompatibleChartsResponse {
                        x_column,

                        y_column,

                        chart_types,
                    },
                )
            },
        )
        .map_err(
            map_chart_error,
        )
}

fn compatible_chart_types(
    x_semantic_type:
        &SemanticType,

    y_semantic_type:
        Option<&SemanticType>,

    x_unique_count:
        Option<usize>,
) -> Vec<String> {
    const CHART_TYPES:
        &[ChartType] = &[
        ChartType::Bar,
        ChartType::Line,
        ChartType::Scatter,
        ChartType::Histogram,
        ChartType::Pie,
    ];

    CHART_TYPES
        .iter()
        .copied()
        .filter(
            |chart_type| {
                chart_is_compatible(
                    *chart_type,

                    x_semantic_type,

                    y_semantic_type,

                    x_unique_count,
                )
            },
        )
        .map(
            |chart_type| {
                chart_type
                    .as_str()
                    .to_string()
            },
        )
        .collect()
}

fn recommendation_response(
    recommendation:
        &ChartRecommendation,
) -> ChartRecommendationResponse {
    ChartRecommendationResponse {
        chart_type:
            recommendation
                .chart_type
                .as_str()
                .to_string(),

        x_column:
            recommendation
                .x_column
                .clone(),

        y_column:
            recommendation
                .y_column
                .clone(),

        confidence:
            chart_confidence_name(
                &recommendation
                    .confidence,
            )
            .to_string(),

        reason:
            recommendation
                .reason
                .clone(),
    }
}

fn chart_confidence_name(
    confidence:
        &ChartConfidence,
) -> &'static str {
    match confidence {
        ChartConfidence::Low => {
            "low"
        }

        ChartConfidence::Medium => {
            "medium"
        }

        ChartConfidence::High => {
            "high"
        }
    }
}

fn map_chart_error(
    error: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_column_supports_histogram() {
        let charts =
            compatible_chart_types(
                &SemanticType::Decimal,

                None,

                None,
            );

        assert_eq!(
            charts,
            vec![
                "histogram".to_string(),
            ]
        );
    }

    #[test]
    fn category_numeric_supports_bar() {
        let charts =
            compatible_chart_types(
                &SemanticType::Category,

                Some(
                    &SemanticType::Currency,
                ),

                Some(10),
            );

        assert!(
            charts.contains(
                &"bar".to_string(),
            )
        );
    }

    #[test]
    fn small_category_numeric_supports_pie() {
        let charts =
            compatible_chart_types(
                &SemanticType::Category,

                Some(
                    &SemanticType::Integer,
                ),

                Some(4),
            );

        assert!(
            charts.contains(
                &"bar".to_string(),
            )
        );

        assert!(
            charts.contains(
                &"pie".to_string(),
            )
        );
    }

    #[test]
    fn date_numeric_supports_line() {
        let charts =
            compatible_chart_types(
                &SemanticType::Date,

                Some(
                    &SemanticType::Decimal,
                ),

                None,
            );

        assert_eq!(
            charts,
            vec![
                "line".to_string(),
            ]
        );
    }

    #[test]
    fn numeric_pair_supports_scatter() {
        let charts =
            compatible_chart_types(
                &SemanticType::Decimal,

                Some(
                    &SemanticType::Integer,
                ),

                None,
            );

        assert_eq!(
            charts,
            vec![
                "scatter".to_string(),
            ]
        );
    }

    #[test]
    fn confidence_names_are_stable() {
        assert_eq!(
            chart_confidence_name(
                &ChartConfidence::Low,
            ),
            "low"
        );

        assert_eq!(
            chart_confidence_name(
                &ChartConfidence::Medium,
            ),
            "medium"
        );

        assert_eq!(
            chart_confidence_name(
                &ChartConfidence::High,
            ),
            "high"
        );
    }
}