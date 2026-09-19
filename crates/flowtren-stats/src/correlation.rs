use polars::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct CorrelationStats {
    /// Jumlah pasangan valid yang benar-benar
    /// digunakan dalam perhitungan.
    pub pair_count: usize,

    /// Pearson correlation coefficient.
    ///
    /// Range:
    /// -1.0 ..= 1.0
    ///
    /// None jika tidak bisa dihitung,
    /// misalnya variance salah satu kolom = 0.
    pub pearson: Option<f64>,
}

pub fn analyze_correlation(
    left: &Series,
    right: &Series,
) -> PolarsResult<CorrelationStats> {
    let pairs =
        numeric_pairs(
            left,
            right,
        )?;

    let pair_count =
        pairs.len();

    if pair_count < 2 {
        return Ok(
            CorrelationStats {
                pair_count,
                pearson: None,
            },
        );
    }

    let left_mean =
        pairs
            .iter()
            .map(|(x, _)| *x)
            .sum::<f64>()
            / pair_count as f64;

    let right_mean =
        pairs
            .iter()
            .map(|(_, y)| *y)
            .sum::<f64>()
            / pair_count as f64;

    let mut covariance_sum =
        0.0;

    let mut left_squared_sum =
        0.0;

    let mut right_squared_sum =
        0.0;

    for (x, y) in &pairs {
        let left_diff =
            x - left_mean;

        let right_diff =
            y - right_mean;

        covariance_sum +=
            left_diff
                * right_diff;

        left_squared_sum +=
            left_diff
                * left_diff;

        right_squared_sum +=
            right_diff
                * right_diff;
    }

    if left_squared_sum == 0.0
        || right_squared_sum == 0.0
    {
        return Ok(
            CorrelationStats {
                pair_count,
                pearson: None,
            },
        );
    }

    let denominator =
        (
            left_squared_sum
                * right_squared_sum
        )
        .sqrt();

    let correlation =
        covariance_sum
            / denominator;

    Ok(
        CorrelationStats {
            pair_count,

            pearson:
                Some(
                    correlation.clamp(
                        -1.0,
                        1.0,
                    ),
                ),
        },
    )
}

pub(crate) fn numeric_pairs(
    left: &Series,
    right: &Series,
) -> PolarsResult<Vec<(f64, f64)>> {
    if left.len()
        != right.len()
    {
        return Err(
            PolarsError::ComputeError(
                "Correlation requires columns with equal length"
                    .into(),
            ),
        );
    }

    let left_casted =
        left.cast(
            &DataType::Float64,
        )?;

    let right_casted =
        right.cast(
            &DataType::Float64,
        )?;

    let left_values =
        left_casted.f64()?;

    let right_values =
        right_casted.f64()?;

    let pairs =
        left_values
            .iter()
            .zip(
                right_values.iter(),
            )
            .filter_map(
                |(left, right)| {
                    match (
                        left,
                        right,
                    ) {
                        (
                            Some(x),
                            Some(y),
                        )
                            if x.is_finite()
                                && y.is_finite() =>
                        {
                            Some((x, y))
                        }

                        _ => None,
                    }
                },
            )
            .collect();

    Ok(pairs)
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
    fn detects_perfect_positive_correlation() {
        let x =
            Series::new(
                "x".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                    5_i64,
                ],
            );

        let y =
            Series::new(
                "y".into(),
                [
                    2_i64,
                    4_i64,
                    6_i64,
                    8_i64,
                    10_i64,
                ],
            );

        let stats =
            analyze_correlation(
                &x,
                &y,
            )
            .expect(
                "Correlation should succeed",
            );

        assert_eq!(
            stats.pair_count,
            5
        );

        assert_close(
            stats.pearson.unwrap(),
            1.0,
        );
    }

    #[test]
    fn detects_perfect_negative_correlation() {
        let x =
            Series::new(
                "x".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                ],
            );

        let y =
            Series::new(
                "y".into(),
                [
                    40_i64,
                    30_i64,
                    20_i64,
                    10_i64,
                ],
            );

        let stats =
            analyze_correlation(
                &x,
                &y,
            )
            .expect(
                "Correlation should succeed",
            );

        assert_close(
            stats.pearson.unwrap(),
            -1.0,
        );
    }

    #[test]
    fn ignores_null_pairs() {
        let x =
            Series::new(
                "x".into(),
                [
                    Some(1_i64),
                    Some(2_i64),
                    None,
                    Some(4_i64),
                    Some(5_i64),
                ],
            );

        let y =
            Series::new(
                "y".into(),
                [
                    Some(2_i64),
                    None,
                    Some(6_i64),
                    Some(8_i64),
                    Some(10_i64),
                ],
            );

        let stats =
            analyze_correlation(
                &x,
                &y,
            )
            .expect(
                "Correlation should succeed",
            );

        // Pasangan valid:
        //
        // 1,2
        // 4,8
        // 5,10
        assert_eq!(
            stats.pair_count,
            3
        );

        assert_close(
            stats.pearson.unwrap(),
            1.0,
        );
    }

    #[test]
    fn constant_column_has_no_correlation() {
        let x =
            Series::new(
                "x".into(),
                [
                    10_i64,
                    10_i64,
                    10_i64,
                    10_i64,
                ],
            );

        let y =
            Series::new(
                "y".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                ],
            );

        let stats =
            analyze_correlation(
                &x,
                &y,
            )
            .expect(
                "Correlation should succeed",
            );

        assert_eq!(
            stats.pearson,
            None
        );
    }

    #[test]
    fn insufficient_pairs_returns_none() {
        let x =
            Series::new(
                "x".into(),
                [
                    Some(1_i64),
                    None,
                ],
            );

        let y =
            Series::new(
                "y".into(),
                [
                    Some(2_i64),
                    Some(3_i64),
                ],
            );

        let stats =
            analyze_correlation(
                &x,
                &y,
            )
            .expect(
                "Correlation should succeed",
            );

        assert_eq!(
            stats.pair_count,
            1
        );

        assert_eq!(
            stats.pearson,
            None
        );
    }

    #[test]
    fn unequal_length_returns_error() {
        let x =
            Series::new(
                "x".into(),
                [
                    1_i64,
                    2_i64,
                ],
            );

        let y =
            Series::new(
                "y".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                ],
            );

        assert!(
            analyze_correlation(
                &x,
                &y,
            )
            .is_err()
        );
    }
}