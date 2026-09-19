use polars::prelude::*;

use crate::correlation::numeric_pairs;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Flat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrendStats {
    pub pair_count: usize,

    pub slope: Option<f64>,
    pub intercept: Option<f64>,

    pub r_squared: Option<f64>,

    pub direction:
        Option<TrendDirection>,
}

pub fn analyze_trend(
    x: &Series,
    y: &Series,
) -> PolarsResult<TrendStats> {
    let pairs =
        numeric_pairs(
            x,
            y,
        )?;

    calculate_trend(
        &pairs,
    )
}

/// Convenience function untuk melihat trend
/// sebuah Series berdasarkan posisi row.
///
/// Contoh:
///
/// value:
/// 10
/// 20
/// 30
///
/// x otomatis:
/// 0
/// 1
/// 2
pub fn analyze_index_trend(
    series: &Series,
) -> PolarsResult<TrendStats> {
    let casted =
        series.cast(
            &DataType::Float64,
        )?;

    let values =
        casted.f64()?;

    let pairs:
        Vec<(f64, f64)> =
        values
            .iter()
            .enumerate()
            .filter_map(
                |(index, value)| {
                    let value =
                        value?;

                    if !value.is_finite() {
                        return None;
                    }

                    Some((
                        index as f64,
                        value,
                    ))
                },
            )
            .collect();

    calculate_trend(
        &pairs,
    )
}

fn calculate_trend(
    pairs: &[(f64, f64)],
) -> PolarsResult<TrendStats> {
    let pair_count =
        pairs.len();

    if pair_count < 2 {
        return Ok(
            TrendStats {
                pair_count,

                slope: None,
                intercept: None,

                r_squared: None,

                direction: None,
            },
        );
    }

    let x_mean =
        pairs
            .iter()
            .map(|(x, _)| *x)
            .sum::<f64>()
            / pair_count as f64;

    let y_mean =
        pairs
            .iter()
            .map(|(_, y)| *y)
            .sum::<f64>()
            / pair_count as f64;

    let mut numerator =
        0.0;

    let mut denominator =
        0.0;

    for (x, y) in pairs {
        let x_diff =
            x - x_mean;

        let y_diff =
            y - y_mean;

        numerator +=
            x_diff
                * y_diff;

        denominator +=
            x_diff
                * x_diff;
    }

    if denominator == 0.0 {
        return Ok(
            TrendStats {
                pair_count,

                slope: None,
                intercept: None,

                r_squared: None,

                direction: None,
            },
        );
    }

    let slope =
        numerator
            / denominator;

    let intercept =
        y_mean
            - slope
                * x_mean;

    let mut total_sum_squares =
        0.0;

    let mut residual_sum_squares =
        0.0;

    for (x, y) in pairs {
        let predicted =
            intercept
                + slope
                    * x;

        let total_diff =
            y - y_mean;

        let residual =
            y - predicted;

        total_sum_squares +=
            total_diff
                * total_diff;

        residual_sum_squares +=
            residual
                * residual;
    }

    let r_squared =
        if total_sum_squares == 0.0 {
            None
        } else {
            Some(
                (
                    1.0
                        - residual_sum_squares
                            / total_sum_squares
                )
                .clamp(
                    0.0,
                    1.0,
                ),
            )
        };

    let direction =
        Some(
            direction_from_slope(
                slope,
            ),
        );

    Ok(
        TrendStats {
            pair_count,

            slope:
                Some(slope),

            intercept:
                Some(intercept),

            r_squared,

            direction,
        },
    )
}

fn direction_from_slope(
    slope: f64,
) -> TrendDirection {
    const FLAT_EPSILON: f64 =
        1e-12;

    if slope.abs()
        <= FLAT_EPSILON
    {
        TrendDirection::Flat
    } else if slope > 0.0 {
        TrendDirection::Increasing
    } else {
        TrendDirection::Decreasing
    }
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
    fn detects_increasing_trend() {
        let x =
            Series::new(
                "time".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                ],
            );

        let y =
            Series::new(
                "sales".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                    40_i64,
                ],
            );

        let stats =
            analyze_trend(
                &x,
                &y,
            )
            .expect(
                "Trend analysis should succeed",
            );

        assert_eq!(
            stats.pair_count,
            4
        );

        assert_close(
            stats.slope.unwrap(),
            10.0,
        );

        assert_close(
            stats.intercept.unwrap(),
            0.0,
        );

        assert_close(
            stats.r_squared.unwrap(),
            1.0,
        );

        assert_eq!(
            stats.direction,
            Some(
                TrendDirection::Increasing
            )
        );
    }

    #[test]
    fn detects_decreasing_trend() {
        let x =
            Series::new(
                "time".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                ],
            );

        let y =
            Series::new(
                "sales".into(),
                [
                    40_i64,
                    30_i64,
                    20_i64,
                    10_i64,
                ],
            );

        let stats =
            analyze_trend(
                &x,
                &y,
            )
            .expect(
                "Trend analysis should succeed",
            );

        assert!(
            stats.slope.unwrap()
                < 0.0
        );

        assert_eq!(
            stats.direction,
            Some(
                TrendDirection::Decreasing
            )
        );

        assert_close(
            stats.r_squared.unwrap(),
            1.0,
        );
    }

    #[test]
    fn detects_flat_trend() {
        let x =
            Series::new(
                "time".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                ],
            );

        let y =
            Series::new(
                "sales".into(),
                [
                    10_i64,
                    10_i64,
                    10_i64,
                    10_i64,
                ],
            );

        let stats =
            analyze_trend(
                &x,
                &y,
            )
            .expect(
                "Trend analysis should succeed",
            );

        assert_eq!(
            stats.slope,
            Some(0.0)
        );

        assert_eq!(
            stats.direction,
            Some(
                TrendDirection::Flat
            )
        );

        // Y tidak punya variance,
        // jadi R² tidak meaningful.
        assert_eq!(
            stats.r_squared,
            None
        );
    }

    #[test]
    fn ignores_null_pairs() {
        let x =
            Series::new(
                "time".into(),
                [
                    Some(1_i64),
                    Some(2_i64),
                    None,
                    Some(4_i64),
                ],
            );

        let y =
            Series::new(
                "sales".into(),
                [
                    Some(10_i64),
                    Some(20_i64),
                    Some(30_i64),
                    Some(40_i64),
                ],
            );

        let stats =
            analyze_trend(
                &x,
                &y,
            )
            .expect(
                "Trend analysis should succeed",
            );

        assert_eq!(
            stats.pair_count,
            3
        );

        assert_close(
            stats.slope.unwrap(),
            10.0,
        );
    }

    #[test]
    fn index_trend_works() {
        let series =
            Series::new(
                "sales".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                    40_i64,
                ],
            );

        let stats =
            analyze_index_trend(
                &series,
            )
            .expect(
                "Index trend should succeed",
            );

        assert_eq!(
            stats.pair_count,
            4
        );

        assert_close(
            stats.slope.unwrap(),
            10.0,
        );

        assert_eq!(
            stats.direction,
            Some(
                TrendDirection::Increasing
            )
        );
    }

    #[test]
    fn index_trend_preserves_original_row_position_with_nulls() {
        let series =
            Series::new(
                "sales".into(),
                [
                    Some(10_i64),
                    None,
                    Some(30_i64),
                ],
            );

        let stats =
            analyze_index_trend(
                &series,
            )
            .expect(
                "Index trend should succeed",
            );

        // Pasangan:
        //
        // x=0, y=10
        // x=2, y=30
        //
        // slope = 10.
        assert_eq!(
            stats.pair_count,
            2
        );

        assert_close(
            stats.slope.unwrap(),
            10.0,
        );
    }

    #[test]
    fn constant_x_cannot_create_trend() {
        let x =
            Series::new(
                "time".into(),
                [
                    1_i64,
                    1_i64,
                    1_i64,
                ],
            );

        let y =
            Series::new(
                "sales".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                ],
            );

        let stats =
            analyze_trend(
                &x,
                &y,
            )
            .expect(
                "Trend analysis should succeed",
            );

        assert_eq!(
            stats.slope,
            None
        );

        assert_eq!(
            stats.direction,
            None
        );
    }

    #[test]
    fn single_pair_has_no_trend() {
        let x =
            Series::new(
                "time".into(),
                [
                    1_i64,
                ],
            );

        let y =
            Series::new(
                "sales".into(),
                [
                    10_i64,
                ],
            );

        let stats =
            analyze_trend(
                &x,
                &y,
            )
            .expect(
                "Trend analysis should succeed",
            );

        assert_eq!(
            stats.pair_count,
            1
        );

        assert_eq!(
            stats.slope,
            None
        );

        assert_eq!(
            stats.r_squared,
            None
        );
    }
}