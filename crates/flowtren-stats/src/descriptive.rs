use polars::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DescriptiveStats {
    /// Total jumlah row, termasuk null.
    pub row_count: usize,

    /// Jumlah value non-null.
    pub count: usize,

    /// Jumlah null.
    pub null_count: usize,

    pub sum: Option<f64>,
    pub mean: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,

    /// Sample standard deviation.
    ///
    /// Menggunakan denominator n - 1.
    /// Untuk count < 2, nilainya None.
    pub std_dev: Option<f64>,
}

pub fn analyze_descriptive(
    series: &Series,
) -> PolarsResult<DescriptiveStats> {
    let values =
        numeric_values(series)?;

    let row_count =
        series.len();

    let null_count =
        series.null_count();

    let count =
        values.len();

    if values.is_empty() {
        return Ok(
            DescriptiveStats {
                row_count,
                count: 0,
                null_count,

                sum: None,
                mean: None,
                min: None,
                max: None,
                std_dev: None,
            },
        );
    }

    let sum: f64 =
        values
            .iter()
            .sum();

    let mean =
        sum / count as f64;

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

    let std_dev =
        sample_std_dev(
            &values,
            mean,
        );

    Ok(
        DescriptiveStats {
            row_count,
            count,
            null_count,

            sum:
                Some(sum),

            mean:
                Some(mean),

            min:
                Some(min),

            max:
                Some(max),

            std_dev,
        },
    )
}

pub(crate) fn numeric_values(
    series: &Series,
) -> PolarsResult<Vec<f64>> {
    let casted =
        series.cast(
            &DataType::Float64,
        )?;

    let values =
        casted.f64()?;

    Ok(
        values
            .iter()
            .flatten()
            .filter(
                |value| {
                    value.is_finite()
                },
            )
            .collect(),
    )
}

fn sample_std_dev(
    values: &[f64],
    mean: f64,
) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }

    let squared_difference_sum:
        f64 =
        values
            .iter()
            .map(
                |value| {
                    let difference =
                        value - mean;

                    difference
                        * difference
                },
            )
            .sum();

    let variance =
        squared_difference_sum
            / (
                values.len()
                    - 1
            ) as f64;

    Some(
        variance.sqrt(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(
        actual: f64,
        expected: f64,
    ) {
        let epsilon =
            1e-10;

        assert!(
            (
                actual
                    - expected
            )
            .abs()
                < epsilon,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn calculates_integer_statistics() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                    40_i64,
                ],
            );

        let stats =
            analyze_descriptive(
                &series,
            )
            .expect(
                "Descriptive statistics should succeed",
            );

        assert_eq!(
            stats.row_count,
            4
        );

        assert_eq!(
            stats.count,
            4
        );

        assert_eq!(
            stats.null_count,
            0
        );

        assert_eq!(
            stats.sum,
            Some(100.0)
        );

        assert_eq!(
            stats.mean,
            Some(25.0)
        );

        assert_eq!(
            stats.min,
            Some(10.0)
        );

        assert_eq!(
            stats.max,
            Some(40.0)
        );

        assert_close(
            stats
                .std_dev
                .unwrap(),
            12.909944487358056,
        );
    }

    #[test]
    fn ignores_null_values() {
        let series =
            Series::new(
                "amount".into(),
                [
                    Some(10_i64),
                    None,
                    Some(30_i64),
                    None,
                ],
            );

        let stats =
            analyze_descriptive(
                &series,
            )
            .expect(
                "Descriptive statistics should succeed",
            );

        assert_eq!(
            stats.row_count,
            4
        );

        assert_eq!(
            stats.count,
            2
        );

        assert_eq!(
            stats.null_count,
            2
        );

        assert_eq!(
            stats.sum,
            Some(40.0)
        );

        assert_eq!(
            stats.mean,
            Some(20.0)
        );

        assert_eq!(
            stats.min,
            Some(10.0)
        );

        assert_eq!(
            stats.max,
            Some(30.0)
        );
    }

    #[test]
    fn handles_float_values() {
        let series =
            Series::new(
                "amount".into(),
                [
                    1.5_f64,
                    2.5_f64,
                    3.5_f64,
                ],
            );

        let stats =
            analyze_descriptive(
                &series,
            )
            .expect(
                "Descriptive statistics should succeed",
            );

        assert_eq!(
            stats.sum,
            Some(7.5)
        );

        assert_eq!(
            stats.mean,
            Some(2.5)
        );

        assert_eq!(
            stats.min,
            Some(1.5)
        );

        assert_eq!(
            stats.max,
            Some(3.5)
        );

        assert_eq!(
            stats.std_dev,
            Some(1.0)
        );
    }

    #[test]
    fn single_value_has_no_sample_std_dev() {
        let series =
            Series::new(
                "amount".into(),
                [
                    100_i64,
                ],
            );

        let stats =
            analyze_descriptive(
                &series,
            )
            .expect(
                "Descriptive statistics should succeed",
            );

        assert_eq!(
            stats.count,
            1
        );

        assert_eq!(
            stats.mean,
            Some(100.0)
        );

        assert_eq!(
            stats.std_dev,
            None
        );
    }

    #[test]
    fn handles_all_null_values() {
        let series =
            Series::new(
                "amount".into(),
                vec![
                    None::<f64>,
                    None,
                    None,
                ],
            );

        let stats =
            analyze_descriptive(
                &series,
            )
            .expect(
                "Descriptive statistics should succeed",
            );

        assert_eq!(
            stats.row_count,
            3
        );

        assert_eq!(
            stats.count,
            0
        );

        assert_eq!(
            stats.null_count,
            3
        );

        assert_eq!(
            stats.sum,
            None
        );

        assert_eq!(
            stats.mean,
            None
        );

        assert_eq!(
            stats.min,
            None
        );

        assert_eq!(
            stats.max,
            None
        );

        assert_eq!(
            stats.std_dev,
            None
        );
    }

    #[test]
    fn handles_empty_series() {
        let series =
            Series::new(
                "amount".into(),
                Vec::<f64>::new(),
            );

        let stats =
            analyze_descriptive(
                &series,
            )
            .expect(
                "Descriptive statistics should succeed",
            );

        assert_eq!(
            stats.row_count,
            0
        );

        assert_eq!(
            stats.count,
            0
        );

        assert_eq!(
            stats.null_count,
            0
        );

        assert_eq!(
            stats.mean,
            None
        );
    }
}