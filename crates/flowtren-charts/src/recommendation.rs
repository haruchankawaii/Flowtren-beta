use flowtren_core::semantic_type::SemanticType;

use crate::{
    chart_type::ChartType,
    compatibility::{
        MAX_PIE_CATEGORIES,
        chart_is_compatible,
        is_numeric_semantic,
        is_temporal_semantic,
    },
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum ChartConfidence {
    Low,
    Medium,
    High,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
)]
pub struct ChartRecommendation {
    pub chart_type:
        ChartType,

    pub x_column:
        String,

    pub y_column:
        Option<String>,

    pub confidence:
        ChartConfidence,

    /// Alasan internal yang juga bisa
    /// dipakai UI untuk menjelaskan
    /// kenapa chart dipilih.
    pub reason:
        String,
}

impl ChartRecommendation {
    pub fn new(
        chart_type: ChartType,
        x_column: impl Into<String>,
        y_column: Option<String>,
        confidence: ChartConfidence,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            chart_type,

            x_column:
                x_column.into(),

            y_column,

            confidence,

            reason:
                reason.into(),
        }
    }
}

pub fn recommend_single_column(
    column_name: &str,
    semantic_type: &SemanticType,
) -> Option<ChartRecommendation> {
    if is_numeric_semantic(
        semantic_type,
    ) {
        return Some(
            ChartRecommendation::new(
                ChartType::Histogram,

                column_name,

                None,

                ChartConfidence::High,

                "Numeric columns are well suited to distribution histograms.",
            ),
        );
    }

    None
}

pub fn recommend_two_columns(
    x_column: &str,
    x_semantic_type: &SemanticType,
    y_column: &str,
    y_semantic_type: &SemanticType,
    x_unique_count: Option<usize>,
) -> Option<ChartRecommendation> {
    // Date / DateTime + numeric
    //
    // Ini prioritas tertinggi karena struktur
    // temporal punya interpretasi yang jelas.
    if is_temporal_semantic(
        x_semantic_type,
    ) && is_numeric_semantic(
        y_semantic_type,
    ) {
        return Some(
            ChartRecommendation::new(
                ChartType::Line,

                x_column,

                Some(
                    y_column.to_string(),
                ),

                ChartConfidence::High,

                "Temporal and numeric columns are suitable for a line chart.",
            ),
        );
    }

    // Numeric + numeric
    if is_numeric_semantic(
        x_semantic_type,
    ) && is_numeric_semantic(
        y_semantic_type,
    ) {
        return Some(
            ChartRecommendation::new(
                ChartType::Scatter,

                x_column,

                Some(
                    y_column.to_string(),
                ),

                ChartConfidence::High,

                "Two numeric columns are suitable for examining their relationship with a scatter plot.",
            ),
        );
    }

    // Category + numeric.
    //
    // Pie hanya untuk cardinality sangat kecil.
    if chart_is_compatible(
        ChartType::Pie,
        x_semantic_type,
        Some(
            y_semantic_type,
        ),
        x_unique_count,
    ) {
        let unique_count =
            x_unique_count
                .unwrap_or(
                    MAX_PIE_CATEGORIES,
                );

        if unique_count <= 4 {
            return Some(
                ChartRecommendation::new(
                    ChartType::Pie,

                    x_column,

                    Some(
                        y_column.to_string(),
                    ),

                    ChartConfidence::Medium,

                    "A small number of categories can be shown as a composition chart.",
                ),
            );
        }
    }

    if chart_is_compatible(
        ChartType::Bar,
        x_semantic_type,
        Some(
            y_semantic_type,
        ),
        x_unique_count,
    ) {
        return Some(
            ChartRecommendation::new(
                ChartType::Bar,

                x_column,

                Some(
                    y_column.to_string(),
                ),

                ChartConfidence::High,

                "Categorical and numeric columns are suitable for comparison with a bar chart.",
            ),
        );
    }

    None
}

pub fn recommend_all_for_pair(
    x_column: &str,
    x_semantic_type: &SemanticType,
    y_column: &str,
    y_semantic_type: &SemanticType,
    x_unique_count: Option<usize>,
) -> Vec<ChartRecommendation> {
    let mut recommendations =
        Vec::new();

    if chart_is_compatible(
        ChartType::Line,
        x_semantic_type,
        Some(
            y_semantic_type,
        ),
        x_unique_count,
    ) {
        recommendations.push(
            ChartRecommendation::new(
                ChartType::Line,

                x_column,

                Some(
                    y_column.to_string(),
                ),

                ChartConfidence::High,

                "Temporal and numeric values can be visualized over time.",
            ),
        );
    }

    if chart_is_compatible(
        ChartType::Scatter,
        x_semantic_type,
        Some(
            y_semantic_type,
        ),
        x_unique_count,
    ) {
        recommendations.push(
            ChartRecommendation::new(
                ChartType::Scatter,

                x_column,

                Some(
                    y_column.to_string(),
                ),

                ChartConfidence::High,

                "Two numeric columns can be compared as coordinate pairs.",
            ),
        );
    }

    if chart_is_compatible(
        ChartType::Bar,
        x_semantic_type,
        Some(
            y_semantic_type,
        ),
        x_unique_count,
    ) {
        recommendations.push(
            ChartRecommendation::new(
                ChartType::Bar,

                x_column,

                Some(
                    y_column.to_string(),
                ),

                ChartConfidence::High,

                "Categories can be compared using bar lengths.",
            ),
        );
    }

    if chart_is_compatible(
        ChartType::Pie,
        x_semantic_type,
        Some(
            y_semantic_type,
        ),
        x_unique_count,
    ) {
        recommendations.push(
            ChartRecommendation::new(
                ChartType::Pie,

                x_column,

                Some(
                    y_column.to_string(),
                ),

                ChartConfidence::Medium,

                "Low-cardinality categories can be shown as part-to-whole composition.",
            ),
        );
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_column_recommends_histogram() {
        let recommendation =
            recommend_single_column(
                "revenue",
                &SemanticType::Currency,
            )
            .expect(
                "Recommendation should exist",
            );

        assert_eq!(
            recommendation.chart_type,
            ChartType::Histogram
        );

        assert_eq!(
            recommendation.confidence,
            ChartConfidence::High
        );
    }

    #[test]
    fn date_and_numeric_recommend_line() {
        let recommendation =
            recommend_two_columns(
                "date",
                &SemanticType::Date,

                "revenue",
                &SemanticType::Currency,

                None,
            )
            .expect(
                "Recommendation should exist",
            );

        assert_eq!(
            recommendation.chart_type,
            ChartType::Line
        );
    }

    #[test]
    fn numeric_pair_recommends_scatter() {
        let recommendation =
            recommend_two_columns(
                "marketing",
                &SemanticType::Decimal,

                "sales",
                &SemanticType::Decimal,

                None,
            )
            .expect(
                "Recommendation should exist",
            );

        assert_eq!(
            recommendation.chart_type,
            ChartType::Scatter
        );
    }

    #[test]
    fn category_numeric_recommends_bar() {
        let recommendation =
            recommend_two_columns(
                "region",
                &SemanticType::Category,

                "sales",
                &SemanticType::Currency,

                Some(10),
            )
            .expect(
                "Recommendation should exist",
            );

        assert_eq!(
            recommendation.chart_type,
            ChartType::Bar
        );
    }

    #[test]
    fn very_small_category_can_recommend_pie() {
        let recommendation =
            recommend_two_columns(
                "status",
                &SemanticType::Category,

                "count",
                &SemanticType::Integer,

                Some(3),
            )
            .expect(
                "Recommendation should exist",
            );

        assert_eq!(
            recommendation.chart_type,
            ChartType::Pie
        );
    }

    #[test]
    fn high_cardinality_identifier_has_no_recommendation() {
        let recommendation =
            recommend_two_columns(
                "customer_id",
                &SemanticType::Identifier,

                "sales",
                &SemanticType::Currency,

                Some(1000),
            );

        assert_eq!(
            recommendation,
            None
        );
    }

    #[test]
    fn returns_multiple_compatible_charts() {
        let recommendations =
            recommend_all_for_pair(
                "region",
                &SemanticType::Category,

                "sales",
                &SemanticType::Currency,

                Some(4),
            );

        assert!(
            recommendations
                .iter()
                .any(
                    |recommendation| {
                        recommendation.chart_type
                            == ChartType::Bar
                    },
                )
        );

        assert!(
            recommendations
                .iter()
                .any(
                    |recommendation| {
                        recommendation.chart_type
                            == ChartType::Pie
                    },
                )
        );
    }

    #[test]
    fn text_pair_does_not_generate_chart() {
        let recommendation =
            recommend_two_columns(
                "description",
                &SemanticType::Text,

                "email",
                &SemanticType::Email,

                None,
            );

        assert_eq!(
            recommendation,
            None
        );
    }
}