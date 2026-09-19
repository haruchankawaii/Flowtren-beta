use std::collections::{
    HashMap,
};

use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct ConsistencyMetrics {
    pub inconsistent_count: usize,
    pub inconsistency_rate: f64,
    pub inconsistent_groups: usize,
}

pub fn analyze_consistency(
    series: &Series,
) -> ConsistencyMetrics {
    if series.dtype()
        != &DataType::String
    {
        return ConsistencyMetrics {
            inconsistent_count: 0,
            inconsistency_rate: 0.0,
            inconsistent_groups: 0,
        };
    }

    let Ok(strings) =
        series.str()
    else {
        return ConsistencyMetrics {
            inconsistent_count: 0,
            inconsistency_rate: 0.0,
            inconsistent_groups: 0,
        };
    };

    let mut groups:
        HashMap<
            String,
            HashMap<String, usize>,
        > =
        HashMap::new();

    let mut valid_count =
        0usize;

    for index in 0..strings.len() {
        let Some(value) =
            strings.get(index)
        else {
            continue;
        };

        let trimmed =
            value.trim();

        if trimmed.is_empty() {
            continue;
        }

        valid_count += 1;

        let key =
            normalize_key(
                trimmed,
            );

        let variants =
            groups
                .entry(key)
                .or_default();

        *variants
            .entry(
                trimmed.to_string(),
            )
            .or_insert(0) += 1;
    }

    let mut inconsistent_count =
        0usize;

    let mut inconsistent_groups =
        0usize;

    for variants in groups.values() {
        if variants.len() <= 1 {
            continue;
        }

        inconsistent_groups += 1;

        let total:
            usize =
            variants
                .values()
                .sum();

        let dominant =
            variants
                .values()
                .copied()
                .max()
                .unwrap_or(0);

        inconsistent_count +=
            total.saturating_sub(
                dominant,
            );
    }

    let inconsistency_rate =
        if valid_count == 0 {
            0.0
        } else {
            inconsistent_count as f64
                / valid_count as f64
        };

    ConsistencyMetrics {
        inconsistent_count,
        inconsistency_rate,
        inconsistent_groups,
    }
}

fn normalize_key(
    value: &str,
) -> String {
    value
        .trim()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_case_inconsistency() {
        let series =
            Series::new(
                "city".into(),
                [
                    "Jakarta",
                    "Jakarta",
                    "Jakarta",
                    "jakarta",
                ],
            );

        let metrics =
            analyze_consistency(
                &series,
            );

        assert_eq!(
            metrics.inconsistent_count,
            1
        );

        assert_eq!(
            metrics.inconsistent_groups,
            1
        );

        assert_eq!(
            metrics.inconsistency_rate,
            0.25
        );
    }

    #[test]
    fn detects_multiple_inconsistent_groups() {
        let series =
            Series::new(
                "status".into(),
                [
                    "Active",
                    "Active",
                    "active",
                    "Pending",
                    "Pending",
                    "pending",
                ],
            );

        let metrics =
            analyze_consistency(
                &series,
            );

        assert_eq!(
            metrics.inconsistent_count,
            2
        );

        assert_eq!(
            metrics.inconsistent_groups,
            2
        );
    }

    #[test]
    fn clean_column_has_zero_inconsistency() {
        let series =
            Series::new(
                "city".into(),
                [
                    "Jakarta",
                    "Bandung",
                    "Surabaya",
                ],
            );

        let metrics =
            analyze_consistency(
                &series,
            );

        assert_eq!(
            metrics.inconsistent_count,
            0
        );

        assert_eq!(
            metrics.inconsistency_rate,
            0.0
        );
    }

    #[test]
    fn numeric_column_has_zero_inconsistency() {
        let series =
            Series::new(
                "amount".into(),
                [
                    10_i64,
                    20_i64,
                    30_i64,
                ],
            );

        let metrics =
            analyze_consistency(
                &series,
            );

        assert_eq!(
            metrics.inconsistent_count,
            0
        );
    }
}