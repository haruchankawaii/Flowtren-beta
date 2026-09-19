use crate::chart_type::ChartType;

#[derive(
    Debug,
    Clone,
    PartialEq,
)]
pub struct CategoryValue {
    pub category: String,
    pub value: f64,
}

impl CategoryValue {
    pub fn new(
        category: impl Into<String>,
        value: f64,
    ) -> Self {
        Self {
            category:
                category.into(),

            value,
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
)]
pub struct XYPoint {
    pub x: f64,
    pub y: f64,
}

impl XYPoint {
    pub fn new(
        x: f64,
        y: f64,
    ) -> Self {
        Self {
            x,
            y,
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
)]
pub struct HistogramBinData {
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub count: usize,
}

impl HistogramBinData {
    pub fn new(
        lower_bound: f64,
        upper_bound: f64,
        count: usize,
    ) -> Self {
        Self {
            lower_bound,
            upper_bound,
            count,
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
)]
pub enum ChartDataset {
    Category {
        values:
            Vec<CategoryValue>,
    },

    XY {
        points:
            Vec<XYPoint>,
    },

    Histogram {
        bins:
            Vec<HistogramBinData>,
    },
}

impl ChartDataset {
    pub fn len(
        &self,
    ) -> usize {
        match self {
            ChartDataset::Category {
                values,
            } => {
                values.len()
            }

            ChartDataset::XY {
                points,
            } => {
                points.len()
            }

            ChartDataset::Histogram {
                bins,
            } => {
                bins.len()
            }
        }
    }

    pub fn is_empty(
        &self,
    ) -> bool {
        self.len() == 0
    }

    pub fn supports_chart(
        &self,
        chart_type: ChartType,
    ) -> bool {
        match self {
            ChartDataset::Category {
                ..
            } => {
                matches!(
                    chart_type,
                    ChartType::Bar
                        | ChartType::Pie
                        | ChartType::Line
                )
            }

            ChartDataset::XY {
                ..
            } => {
                matches!(
                    chart_type,
                    ChartType::Scatter
                        | ChartType::Line
                )
            }

            ChartDataset::Histogram {
                ..
            } => {
                chart_type
                    == ChartType::Histogram
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_dataset_reports_length() {
        let dataset =
            ChartDataset::Category {
                values:
                    vec![
                        CategoryValue::new(
                            "A",
                            10.0,
                        ),
                        CategoryValue::new(
                            "B",
                            20.0,
                        ),
                    ],
            };

        assert_eq!(
            dataset.len(),
            2
        );

        assert!(
            !dataset.is_empty()
        );
    }

    #[test]
    fn histogram_dataset_supports_histogram_chart() {
        let dataset =
            ChartDataset::Histogram {
                bins:
                    vec![
                        HistogramBinData::new(
                            0.0,
                            10.0,
                            5,
                        ),
                    ],
            };

        assert!(
            dataset.supports_chart(
                ChartType::Histogram,
            )
        );

        assert!(
            !dataset.supports_chart(
                ChartType::Scatter,
            )
        );
    }

    #[test]
    fn xy_dataset_supports_scatter() {
        let dataset =
            ChartDataset::XY {
                points:
                    vec![
                        XYPoint::new(
                            1.0,
                            2.0,
                        ),
                    ],
            };

        assert!(
            dataset.supports_chart(
                ChartType::Scatter,
            )
        );
    }
}