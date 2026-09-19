#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum ChartType {
    Bar,
    Line,
    Scatter,
    Histogram,
    Pie,
}

impl ChartType {
    pub fn as_str(
        &self,
    ) -> &'static str {
        match self {
            ChartType::Bar => {
                "bar"
            }

            ChartType::Line => {
                "line"
            }

            ChartType::Scatter => {
                "scatter"
            }

            ChartType::Histogram => {
                "histogram"
            }

            ChartType::Pie => {
                "pie"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_type_has_stable_string_name() {
        assert_eq!(
            ChartType::Bar.as_str(),
            "bar"
        );

        assert_eq!(
            ChartType::Line.as_str(),
            "line"
        );

        assert_eq!(
            ChartType::Scatter.as_str(),
            "scatter"
        );

        assert_eq!(
            ChartType::Histogram.as_str(),
            "histogram"
        );

        assert_eq!(
            ChartType::Pie.as_str(),
            "pie"
        );
    }
}