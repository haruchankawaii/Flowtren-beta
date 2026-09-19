use flowtren_core::semantic_type::SemanticType;

use crate::chart_type::ChartType;

pub const MAX_PIE_CATEGORIES: usize =
    6;

pub const MAX_RECOMMENDED_BAR_CATEGORIES:
    usize = 30;

pub fn is_numeric_semantic(
    semantic_type: &SemanticType,
) -> bool {
    matches!(
        semantic_type,
        SemanticType::Integer
            | SemanticType::Decimal
            | SemanticType::Currency
            | SemanticType::Percentage
    )
}

pub fn is_temporal_semantic(
    semantic_type: &SemanticType,
) -> bool {
    matches!(
        semantic_type,
        SemanticType::Date
            | SemanticType::DateTime
    )
}

pub fn is_categorical_semantic(
    semantic_type: &SemanticType,
) -> bool {
    matches!(
        semantic_type,
        SemanticType::Category
            | SemanticType::Boolean
    )
}

pub fn chart_is_compatible(
    chart_type: ChartType,
    x_semantic_type: &SemanticType,
    y_semantic_type: Option<&SemanticType>,
    category_count: Option<usize>,
) -> bool {
    match chart_type {
        ChartType::Histogram => {
            y_semantic_type.is_none()
                && is_numeric_semantic(
                    x_semantic_type,
                )
        }

        ChartType::Bar => {
            let Some(y_type) =
                y_semantic_type
            else {
                return false;
            };

            let category_ok =
                is_categorical_semantic(
                    x_semantic_type,
                )
                    || matches!(
                        x_semantic_type,
                        SemanticType::Text
                    );

            let cardinality_ok =
                category_count
                    .map(
                        |count| {
                            count
                                <= MAX_RECOMMENDED_BAR_CATEGORIES
                        },
                    )
                    .unwrap_or(true);

            category_ok
                && cardinality_ok
                && is_numeric_semantic(
                    y_type,
                )
        }

        ChartType::Pie => {
            let Some(y_type) =
                y_semantic_type
            else {
                return false;
            };

            let category_ok =
                is_categorical_semantic(
                    x_semantic_type,
                );

            let cardinality_ok =
                category_count
                    .map(
                        |count| {
                            count >= 2
                                && count
                                    <= MAX_PIE_CATEGORIES
                        },
                    )
                    .unwrap_or(false);

            category_ok
                && cardinality_ok
                && is_numeric_semantic(
                    y_type,
                )
        }

        ChartType::Line => {
            let Some(y_type) =
                y_semantic_type
            else {
                return false;
            };

            is_temporal_semantic(
                x_semantic_type,
            )
                && is_numeric_semantic(
                    y_type,
                )
        }

        ChartType::Scatter => {
            let Some(y_type) =
                y_semantic_type
            else {
                return false;
            };

            is_numeric_semantic(
                x_semantic_type,
            )
                && is_numeric_semantic(
                    y_type,
                )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_column_supports_histogram() {
        assert!(
            chart_is_compatible(
                ChartType::Histogram,
                &SemanticType::Decimal,
                None,
                None,
            )
        );
    }

    #[test]
    fn category_and_numeric_support_bar() {
        assert!(
            chart_is_compatible(
                ChartType::Bar,
                &SemanticType::Category,
                Some(
                    &SemanticType::Currency,
                ),
                Some(5),
            )
        );
    }

    #[test]
    fn too_many_categories_reject_bar() {
        assert!(
            !chart_is_compatible(
                ChartType::Bar,
                &SemanticType::Category,
                Some(
                    &SemanticType::Decimal,
                ),
                Some(100),
            )
        );
    }

    #[test]
    fn date_and_numeric_support_line() {
        assert!(
            chart_is_compatible(
                ChartType::Line,
                &SemanticType::Date,
                Some(
                    &SemanticType::Decimal,
                ),
                None,
            )
        );
    }

    #[test]
    fn two_numeric_columns_support_scatter() {
        assert!(
            chart_is_compatible(
                ChartType::Scatter,
                &SemanticType::Decimal,
                Some(
                    &SemanticType::Integer,
                ),
                None,
            )
        );
    }

    #[test]
    fn pie_requires_low_cardinality() {
        assert!(
            chart_is_compatible(
                ChartType::Pie,
                &SemanticType::Category,
                Some(
                    &SemanticType::Integer,
                ),
                Some(5),
            )
        );

        assert!(
            !chart_is_compatible(
                ChartType::Pie,
                &SemanticType::Category,
                Some(
                    &SemanticType::Integer,
                ),
                Some(20),
            )
        );
    }

    #[test]
    fn identifier_is_not_treated_as_category() {
        assert!(
            !chart_is_compatible(
                ChartType::Bar,
                &SemanticType::Identifier,
                Some(
                    &SemanticType::Decimal,
                ),
                Some(10),
            )
        );
    }
}