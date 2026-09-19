use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct CardinalityMetrics {
    pub unique_count: usize,
    pub unique_rate: f64,
}

pub fn analyze_cardinality(
    series: &Series,
) -> PolarsResult<CardinalityMetrics> {
    let row_count =
        series.len();

    if row_count == 0 {
        return Ok(
            CardinalityMetrics {
                unique_count: 0,
                unique_rate: 0.0,
            },
        );
    }

    let total_unique =
        series.n_unique()?;

    // Polars menghitung null sebagai satu unique value.
    // Untuk quality report, kita ingin unique non-null.
    let unique_count =
        if series.null_count() > 0 {
            total_unique.saturating_sub(1)
        } else {
            total_unique
        };

    let non_null_count =
        row_count
            .saturating_sub(
                series.null_count(),
            );

    let unique_rate =
        if non_null_count == 0 {
            0.0
        } else {
            unique_count as f64
                / non_null_count as f64
        };

    Ok(
        CardinalityMetrics {
            unique_count,
            unique_rate,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_cardinality() {
        let series =
            Series::new(
                "city".into(),
                [
                    Some("Jakarta"),
                    Some("Jakarta"),
                    Some("Bandung"),
                    None,
                ],
            );

        let metrics =
            analyze_cardinality(
                &series,
            )
            .expect(
                "Cardinality analysis should succeed",
            );

        assert_eq!(
            metrics.unique_count,
            2
        );

        assert_eq!(
            metrics.unique_rate,
            2.0 / 3.0
        );
    }

    #[test]
    fn all_unique_values_have_rate_one() {
        let series =
            Series::new(
                "id".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                ],
            );

        let metrics =
            analyze_cardinality(
                &series,
            )
            .expect(
                "Cardinality analysis should succeed",
            );

        assert_eq!(
            metrics.unique_count,
            3
        );

        assert_eq!(
            metrics.unique_rate,
            1.0
        );
    }

    #[test]
    fn handles_all_null_column() {
        let series =
            Series::new(
                "value".into(),
                vec![
                    None::<i64>,
                    None,
                    None,
                ],
            );

        let metrics =
            analyze_cardinality(
                &series,
            )
            .expect(
                "Cardinality analysis should succeed",
            );

        assert_eq!(
            metrics.unique_count,
            0
        );

        assert_eq!(
            metrics.unique_rate,
            0.0
        );
    }
}