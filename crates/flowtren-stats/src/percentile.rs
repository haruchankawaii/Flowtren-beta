use polars::prelude::*;

use crate::descriptive::numeric_values;

#[derive(Debug, Clone, PartialEq)]
pub struct PercentileStats {
    pub p25: Option<f64>,
    pub median: Option<f64>,
    pub p75: Option<f64>,
}

pub fn analyze_percentiles(
    series: &Series,
) -> PolarsResult<PercentileStats> {
    let mut values =
        numeric_values(
            series,
        )?;

    if values.is_empty() {
        return Ok(
            PercentileStats {
                p25: None,
                median: None,
                p75: None,
            },
        );
    }

    values.sort_by(
        |a, b| {
            a.total_cmp(b)
        },
    );

    Ok(
        PercentileStats {
            p25:
                percentile_sorted(
                    &values,
                    0.25,
                ),

            median:
                percentile_sorted(
                    &values,
                    0.50,
                ),

            p75:
                percentile_sorted(
                    &values,
                    0.75,
                ),
        },
    )
}

pub fn percentile(
    series: &Series,
    percentile: f64,
) -> PolarsResult<Option<f64>> {
    let mut values =
        numeric_values(
            series,
        )?;

    if values.is_empty() {
        return Ok(None);
    }

    values.sort_by(
        |a, b| {
            a.total_cmp(b)
        },
    );

    Ok(
        percentile_sorted(
            &values,
            percentile,
        ),
    )
}

pub(crate) fn percentile_sorted(
    sorted_values: &[f64],
    percentile: f64,
) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }

    let percentile =
        percentile.clamp(
            0.0,
            1.0,
        );

    if sorted_values.len() == 1 {
        return Some(
            sorted_values[0],
        );
    }

    let position =
        percentile
            * (
                sorted_values.len()
                    - 1
            ) as f64;

    let lower_index =
        position.floor()
            as usize;

    let upper_index =
        position.ceil()
            as usize;

    if lower_index
        == upper_index
    {
        return Some(
            sorted_values[
                lower_index
            ],
        );
    }

    let lower =
        sorted_values[
            lower_index
        ];

    let upper =
        sorted_values[
            upper_index
        ];

    let fraction =
        position
            - lower_index
                as f64;

    Some(
        lower
            + (
                upper
                    - lower
            ) * fraction,
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
    fn calculates_quartiles() {
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
            analyze_percentiles(
                &series,
            )
            .expect(
                "Percentile analysis should succeed",
            );

        assert_close(
            stats.p25.unwrap(),
            17.5,
        );

        assert_close(
            stats.median.unwrap(),
            25.0,
        );

        assert_close(
            stats.p75.unwrap(),
            32.5,
        );
    }

    #[test]
    fn calculates_odd_median() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                    40_i64,
                    50_i64,
                ],
            );

        let stats =
            analyze_percentiles(
                &series,
            )
            .expect(
                "Percentile analysis should succeed",
            );

        assert_eq!(
            stats.median,
            Some(30.0)
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
                    Some(20_i64),
                    Some(30_i64),
                    None,
                    Some(40_i64),
                ],
            );

        let stats =
            analyze_percentiles(
                &series,
            )
            .expect(
                "Percentile analysis should succeed",
            );

        assert_close(
            stats.median.unwrap(),
            25.0,
        );
    }

    #[test]
    fn calculates_custom_percentile() {
        let series =
            Series::new(
                "amount".into(),
                [
                    0_i64,
                    25_i64,
                    50_i64,
                    75_i64,
                    100_i64,
                ],
            );

        let result =
            percentile(
                &series,
                0.90,
            )
            .expect(
                "Percentile should succeed",
            )
            .expect(
                "Percentile should exist",
            );

        assert_close(
            result,
            90.0,
        );
    }

    #[test]
    fn clamps_percentile_below_zero() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                ],
            );

        let result =
            percentile(
                &series,
                -10.0,
            )
            .unwrap();

        assert_eq!(
            result,
            Some(10.0)
        );
    }

    #[test]
    fn clamps_percentile_above_one() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                ],
            );

        let result =
            percentile(
                &series,
                10.0,
            )
            .unwrap();

        assert_eq!(
            result,
            Some(30.0)
        );
    }

    #[test]
    fn handles_single_value() {
        let series =
            Series::new(
                "amount".into(),
                [
                    42_i64,
                ],
            );

        let stats =
            analyze_percentiles(
                &series,
            )
            .expect(
                "Percentile analysis should succeed",
            );

        assert_eq!(
            stats.p25,
            Some(42.0)
        );

        assert_eq!(
            stats.median,
            Some(42.0)
        );

        assert_eq!(
            stats.p75,
            Some(42.0)
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
            analyze_percentiles(
                &series,
            )
            .expect(
                "Percentile analysis should succeed",
            );

        assert_eq!(
            stats.p25,
            None
        );

        assert_eq!(
            stats.median,
            None
        );

        assert_eq!(
            stats.p75,
            None
        );
    }

    #[test]
    fn sorts_unsorted_input() {
        let series =
            Series::new(
                "amount".into(),
                [
                    40_i64,
                    10_i64,
                    30_i64,
                    20_i64,
                ],
            );

        let stats =
            analyze_percentiles(
                &series,
            )
            .expect(
                "Percentile analysis should succeed",
            );

        assert_eq!(
            stats.median,
            Some(25.0)
        );
    }
}