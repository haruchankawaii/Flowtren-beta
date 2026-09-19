use polars::prelude::*;

use crate::{
    descriptive::numeric_values,
    percentile::percentile_sorted,
};

const DEFAULT_IQR_MULTIPLIER: f64 =
    1.5;

#[derive(Debug, Clone, PartialEq)]
pub struct OutlierStats {
    pub count: usize,

    pub q1: Option<f64>,
    pub q3: Option<f64>,

    pub iqr: Option<f64>,

    pub lower_bound: Option<f64>,
    pub upper_bound: Option<f64>,

    pub outlier_count: usize,
    pub outlier_rate: f64,

    pub lower_outlier_count: usize,
    pub upper_outlier_count: usize,
}

pub fn analyze_outliers(
    series: &Series,
) -> PolarsResult<OutlierStats> {
    analyze_outliers_with_multiplier(
        series,
        DEFAULT_IQR_MULTIPLIER,
    )
}

pub fn analyze_outliers_with_multiplier(
    series: &Series,
    multiplier: f64,
) -> PolarsResult<OutlierStats> {
    let mut values =
        numeric_values(series)?;

    let count =
        values.len();

    if values.is_empty() {
        return Ok(
            OutlierStats {
                count: 0,

                q1: None,
                q3: None,

                iqr: None,

                lower_bound: None,
                upper_bound: None,

                outlier_count: 0,
                outlier_rate: 0.0,

                lower_outlier_count: 0,
                upper_outlier_count: 0,
            },
        );
    }

    values.sort_by(
        |a, b| {
            a.total_cmp(b)
        },
    );

    let q1 =
        percentile_sorted(
            &values,
            0.25,
        )
        .expect(
            "Non-empty values must have Q1",
        );

    let q3 =
        percentile_sorted(
            &values,
            0.75,
        )
        .expect(
            "Non-empty values must have Q3",
        );

    let iqr =
        q3 - q1;

    let multiplier =
        multiplier.max(0.0);

    let lower_bound =
        q1
            - multiplier
                * iqr;

    let upper_bound =
        q3
            + multiplier
                * iqr;

    let lower_outlier_count =
        values
            .iter()
            .filter(
                |value| {
                    **value
                        < lower_bound
                },
            )
            .count();

    let upper_outlier_count =
        values
            .iter()
            .filter(
                |value| {
                    **value
                        > upper_bound
                },
            )
            .count();

    let outlier_count =
        lower_outlier_count
            + upper_outlier_count;

    let outlier_rate =
        outlier_count as f64
            / count as f64;

    Ok(
        OutlierStats {
            count,

            q1:
                Some(q1),

            q3:
                Some(q3),

            iqr:
                Some(iqr),

            lower_bound:
                Some(lower_bound),

            upper_bound:
                Some(upper_bound),

            outlier_count,
            outlier_rate,

            lower_outlier_count,
            upper_outlier_count,
        },
    )
}

pub fn is_outlier(
    value: f64,
    stats: &OutlierStats,
) -> bool {
    let (
        Some(lower_bound),
        Some(upper_bound),
    ) =
        (
            stats.lower_bound,
            stats.upper_bound,
        )
    else {
        return false;
    };

    value < lower_bound
        || value > upper_bound
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
    fn detects_upper_outlier() {
        let series =
            Series::new(
                "amount".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                    100_i64,
                ],
            );

        let stats =
            analyze_outliers(
                &series,
            )
            .expect(
                "Outlier analysis should succeed",
            );

        assert_eq!(
            stats.count,
            5
        );

        assert_eq!(
            stats.q1,
            Some(2.0)
        );

        assert_eq!(
            stats.q3,
            Some(4.0)
        );

        assert_eq!(
            stats.iqr,
            Some(2.0)
        );

        assert_eq!(
            stats.lower_bound,
            Some(-1.0)
        );

        assert_eq!(
            stats.upper_bound,
            Some(7.0)
        );

        assert_eq!(
            stats.outlier_count,
            1
        );

        assert_eq!(
            stats.lower_outlier_count,
            0
        );

        assert_eq!(
            stats.upper_outlier_count,
            1
        );

        assert_close(
            stats.outlier_rate,
            0.2,
        );
    }

    #[test]
    fn detects_lower_outlier() {
        let series =
            Series::new(
                "amount".into(),
                [
                    -100_i64,
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                ],
            );

        let stats =
            analyze_outliers(
                &series,
            )
            .expect(
                "Outlier analysis should succeed",
            );

        assert_eq!(
            stats.outlier_count,
            1
        );

        assert_eq!(
            stats.lower_outlier_count,
            1
        );

        assert_eq!(
            stats.upper_outlier_count,
            0
        );
    }

    #[test]
    fn clean_values_have_no_outliers() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    11_i64,
                    12_i64,
                    13_i64,
                    14_i64,
                ],
            );

        let stats =
            analyze_outliers(
                &series,
            )
            .expect(
                "Outlier analysis should succeed",
            );

        assert_eq!(
            stats.outlier_count,
            0
        );

        assert_eq!(
            stats.outlier_rate,
            0.0
        );
    }

    #[test]
    fn ignores_null_values() {
        let series =
            Series::new(
                "amount".into(),
                [
                    Some(1_i64),
                    None,
                    Some(2_i64),
                    Some(3_i64),
                    None,
                    Some(4_i64),
                    Some(100_i64),
                ],
            );

        let stats =
            analyze_outliers(
                &series,
            )
            .expect(
                "Outlier analysis should succeed",
            );

        assert_eq!(
            stats.count,
            5
        );

        assert_eq!(
            stats.outlier_count,
            1
        );

        assert_close(
            stats.outlier_rate,
            0.2,
        );
    }

    #[test]
    fn constant_values_have_no_outliers() {
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

        let stats =
            analyze_outliers(
                &series,
            )
            .expect(
                "Outlier analysis should succeed",
            );

        assert_eq!(
            stats.iqr,
            Some(0.0)
        );

        assert_eq!(
            stats.lower_bound,
            Some(10.0)
        );

        assert_eq!(
            stats.upper_bound,
            Some(10.0)
        );

        assert_eq!(
            stats.outlier_count,
            0
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
            analyze_outliers(
                &series,
            )
            .expect(
                "Outlier analysis should succeed",
            );

        assert_eq!(
            stats.count,
            1
        );

        assert_eq!(
            stats.q1,
            Some(42.0)
        );

        assert_eq!(
            stats.q3,
            Some(42.0)
        );

        assert_eq!(
            stats.iqr,
            Some(0.0)
        );

        assert_eq!(
            stats.outlier_count,
            0
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
            analyze_outliers(
                &series,
            )
            .expect(
                "Outlier analysis should succeed",
            );

        assert_eq!(
            stats.count,
            0
        );

        assert_eq!(
            stats.q1,
            None
        );

        assert_eq!(
            stats.q3,
            None
        );

        assert_eq!(
            stats.iqr,
            None
        );

        assert_eq!(
            stats.lower_bound,
            None
        );

        assert_eq!(
            stats.upper_bound,
            None
        );

        assert_eq!(
            stats.outlier_count,
            0
        );
    }

    #[test]
    fn helper_detects_outlier() {
        let series =
            Series::new(
                "amount".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                    100_i64,
                ],
            );

        let stats =
            analyze_outliers(
                &series,
            )
            .expect(
                "Outlier analysis should succeed",
            );

        assert!(
            is_outlier(
                100.0,
                &stats,
            )
        );

        assert!(
            !is_outlier(
                3.0,
                &stats,
            )
        );
    }

    #[test]
    fn supports_custom_multiplier() {
        let series =
            Series::new(
                "amount".into(),
                [
                    1_i64,
                    2_i64,
                    3_i64,
                    4_i64,
                    10_i64,
                ],
            );

        let normal =
            analyze_outliers_with_multiplier(
                &series,
                1.5,
            )
            .unwrap();

        let stricter =
            analyze_outliers_with_multiplier(
                &series,
                0.5,
            )
            .unwrap();

        assert!(
            stricter.outlier_count
                >= normal.outlier_count
        );
    }
}