use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct DuplicateMetrics {
    pub duplicate_count: usize,
    pub duplicate_rate: f64,
}

pub fn analyze_duplicates(
    df: &DataFrame,
) -> PolarsResult<DuplicateMetrics> {
    let row_count =
        df.height();

    if row_count == 0 {
        return Ok(
            DuplicateMetrics {
                duplicate_count: 0,
                duplicate_rate: 0.0,
            },
        );
    }

    let unique =
        df.unique_stable(
            None,
            UniqueKeepStrategy::First,
            None,
        )?;

    let duplicate_count =
        row_count
            .saturating_sub(
                unique.height(),
            );

    let duplicate_rate =
        duplicate_count as f64
            / row_count as f64;

    Ok(
        DuplicateMetrics {
            duplicate_count,
            duplicate_rate,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_duplicate_rows() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            1_i64,
                            2_i64,
                            3_i64,
                            3_i64,
                        ],
                    ),
                    Column::new(
                        "city".into(),
                        vec![
                            "Jakarta",
                            "Jakarta",
                            "Bandung",
                            "Surabaya",
                            "Surabaya",
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let metrics =
            analyze_duplicates(
                &df,
            )
            .expect(
                "Duplicate analysis should succeed",
            );

        assert_eq!(
            metrics.duplicate_count,
            2
        );

        assert_eq!(
            metrics.duplicate_rate,
            0.4
        );
    }

    #[test]
    fn handles_no_duplicates() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            2_i64,
                            3_i64,
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let metrics =
            analyze_duplicates(
                &df,
            )
            .expect(
                "Duplicate analysis should succeed",
            );

        assert_eq!(
            metrics.duplicate_count,
            0
        );

        assert_eq!(
            metrics.duplicate_rate,
            0.0
        );
    }
}