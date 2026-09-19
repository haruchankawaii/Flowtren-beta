use polars::prelude::*;

use crate::descriptive::numeric_values;

const DEFAULT_BIN_COUNT: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBin {
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistributionStats {
    pub count: usize,

    pub min: Option<f64>,
    pub max: Option<f64>,
    pub range: Option<f64>,

    pub bin_count: usize,

    pub bins: Vec<HistogramBin>,
}

pub fn analyze_distribution(
    series: &Series,
) -> PolarsResult<DistributionStats> {
    analyze_distribution_with_bins(
        series,
        DEFAULT_BIN_COUNT,
    )
}

pub fn analyze_distribution_with_bins(
    series: &Series,
    requested_bin_count: usize,
) -> PolarsResult<DistributionStats> {
    let values =
        numeric_values(series)?;

    let count =
        values.len();

    if values.is_empty() {
        return Ok(
            DistributionStats {
                count: 0,

                min: None,
                max: None,
                range: None,

                bin_count: 0,

                bins: Vec::new(),
            },
        );
    }

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

    let range =
        max - min;

    // Kalau semua value sama:
    //
    // [10, 10, 10, 10]
    //
    // tidak ada gunanya membuat 10 bin kosong.
    // Kita cukup punya satu bin.
    if range == 0.0 {
        return Ok(
            DistributionStats {
                count,

                min:
                    Some(min),

                max:
                    Some(max),

                range:
                    Some(0.0),

                bin_count:
                    1,

                bins:
                    vec![
                        HistogramBin {
                            lower_bound:
                                min,

                            upper_bound:
                                max,

                            count,
                        },
                    ],
            },
        );
    }

    let bin_count =
        requested_bin_count
            .max(1)
            .min(count);

    let bin_width =
        range
            / bin_count as f64;

    let mut counts =
        vec![
            0usize;
            bin_count
        ];

    for value in &values {
        let mut index =
            (
                (
                    value - min
                )
                / bin_width
            )
            .floor()
                as usize;

        // Nilai maksimum menghasilkan index == bin_count.
        //
        // Contoh:
        //
        // min = 0
        // max = 100
        // bins = 10
        //
        // 100 / 10 = index 10
        //
        // padahal index valid terakhir adalah 9.
        if index >= bin_count {
            index =
                bin_count - 1;
        }

        counts[index] += 1;
    }

    let bins =
        (0..bin_count)
            .map(|index| {
                let lower_bound =
                    min
                        + bin_width
                            * index as f64;

                let upper_bound =
                    if index
                        == bin_count - 1
                    {
                        max
                    } else {
                        min
                            + bin_width
                                * (
                                    index + 1
                                ) as f64
                    };

                HistogramBin {
                    lower_bound,
                    upper_bound,

                    count:
                        counts[index],
                }
            })
            .collect();

    Ok(
        DistributionStats {
            count,

            min:
                Some(min),

            max:
                Some(max),

            range:
                Some(range),

            bin_count,

            bins,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_distribution() {
        let series =
            Series::new(
                "amount".into(),
                [
                    0_i64,
                    10_i64,
                    20_i64,
                    30_i64,
                    40_i64,
                    50_i64,
                    60_i64,
                    70_i64,
                    80_i64,
                    90_i64,
                    100_i64,
                ],
            );

        let distribution =
            analyze_distribution_with_bins(
                &series,
                5,
            )
            .expect(
                "Distribution analysis should succeed",
            );

        assert_eq!(
            distribution.count,
            11
        );

        assert_eq!(
            distribution.min,
            Some(0.0)
        );

        assert_eq!(
            distribution.max,
            Some(100.0)
        );

        assert_eq!(
            distribution.range,
            Some(100.0)
        );

        assert_eq!(
            distribution.bin_count,
            5
        );

        assert_eq!(
            distribution.bins.len(),
            5
        );

        let total_count:
            usize =
            distribution
                .bins
                .iter()
                .map(
                    |bin| {
                        bin.count
                    },
                )
                .sum();

        assert_eq!(
            total_count,
            11
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
                    None,
                    Some(30_i64),
                ],
            );

        let distribution =
            analyze_distribution_with_bins(
                &series,
                3,
            )
            .expect(
                "Distribution analysis should succeed",
            );

        assert_eq!(
            distribution.count,
            3
        );

        let total_count:
            usize =
            distribution
                .bins
                .iter()
                .map(
                    |bin| {
                        bin.count
                    },
                )
                .sum();

        assert_eq!(
            total_count,
            3
        );
    }

    #[test]
    fn constant_values_use_single_bin() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    10_i64,
                    10_i64,
                    10_i64,
                ],
            );

        let distribution =
            analyze_distribution(
                &series,
            )
            .expect(
                "Distribution analysis should succeed",
            );

        assert_eq!(
            distribution.bin_count,
            1
        );

        assert_eq!(
            distribution.bins.len(),
            1
        );

        assert_eq!(
            distribution.bins[0].count,
            4
        );

        assert_eq!(
            distribution.range,
            Some(0.0)
        );
    }

    #[test]
    fn zero_requested_bins_becomes_one() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                ],
            );

        let distribution =
            analyze_distribution_with_bins(
                &series,
                0,
            )
            .expect(
                "Distribution analysis should succeed",
            );

        assert_eq!(
            distribution.bin_count,
            1
        );

        assert_eq!(
            distribution.bins.len(),
            1
        );

        assert_eq!(
            distribution.bins[0].count,
            3
        );
    }

    #[test]
    fn does_not_create_more_bins_than_values() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                ],
            );

        let distribution =
            analyze_distribution_with_bins(
                &series,
                100,
            )
            .expect(
                "Distribution analysis should succeed",
            );

        assert_eq!(
            distribution.bin_count,
            3
        );
    }

    #[test]
    fn handles_empty_series() {
        let series =
            Series::new(
                "amount".into(),
                Vec::<f64>::new(),
            );

        let distribution =
            analyze_distribution(
                &series,
            )
            .expect(
                "Distribution analysis should succeed",
            );

        assert_eq!(
            distribution.count,
            0
        );

        assert_eq!(
            distribution.min,
            None
        );

        assert_eq!(
            distribution.max,
            None
        );

        assert_eq!(
            distribution.range,
            None
        );

        assert_eq!(
            distribution.bin_count,
            0
        );

        assert!(
            distribution
                .bins
                .is_empty()
        );
    }

    #[test]
    fn maximum_value_goes_into_last_bin() {
        let series =
            Series::new(
                "amount".into(),
                [
                    0_i64,
                    50_i64,
                    100_i64,
                ],
            );

        let distribution =
            analyze_distribution_with_bins(
                &series,
                2,
            )
            .expect(
                "Distribution analysis should succeed",
            );

        assert_eq!(
            distribution.bins.len(),
            2
        );

        assert_eq!(
            distribution.bins[1].count,
            2
        );
    }
}