use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct MissingMetrics {
    pub missing_count: usize,
    pub missing_rate: f64,
}

pub fn analyze_missing(
    series: &Series,
) -> MissingMetrics {
    let row_count =
        series.len();

    let missing_count =
        series.null_count();

    let missing_rate =
        if row_count == 0 {
            0.0
        } else {
            missing_count as f64
                / row_count as f64
        };

    MissingMetrics {
        missing_count,
        missing_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_missing_metrics() {
        let series =
            Series::new(
                "age".into(),
                [
                    Some(20_i64),
                    None,
                    Some(30_i64),
                    None,
                ],
            );

        let metrics =
            analyze_missing(
                &series,
            );

        assert_eq!(
            metrics.missing_count,
            2
        );

        assert_eq!(
            metrics.missing_rate,
            0.5
        );
    }

    #[test]
    fn handles_no_missing_values() {
        let series =
            Series::new(
                "age".into(),
                [
                    20_i64,
                    30_i64,
                    40_i64,
                ],
            );

        let metrics =
            analyze_missing(
                &series,
            );

        assert_eq!(
            metrics.missing_count,
            0
        );

        assert_eq!(
            metrics.missing_rate,
            0.0
        );
    }

    #[test]
    fn handles_empty_series() {
        let series =
            Series::new(
                "age".into(),
                Vec::<Option<i64>>::new(),
            );

        let metrics =
            analyze_missing(
                &series,
            );

        assert_eq!(
            metrics.missing_count,
            0
        );

        assert_eq!(
            metrics.missing_rate,
            0.0
        );
    }
}