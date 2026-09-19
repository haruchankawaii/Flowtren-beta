use polars::prelude::*;

use crate::suggestion::{
    CleaningKind,
    CleaningSuggestion,
};

pub fn analyze_duplicates(
    df: &DataFrame,
) -> PolarsResult<Option<CleaningSuggestion>> {
    if df.height() == 0 {
        return Ok(None);
    }

    let unique_df = df.unique_stable(
        None,
        UniqueKeepStrategy::First,
        None,
    )?;

    let duplicated_rows = df
        .height()
        .saturating_sub(
            unique_df.height(),
        );

    if duplicated_rows == 0 {
        return Ok(None);
    }

    Ok(Some(
        CleaningSuggestion::new(
            "__row__",
            CleaningKind::RemoveDuplicateRows,
            duplicated_rows,
            1.0,
        ),
    ))
}

pub fn apply_remove_duplicates(
    df: &DataFrame,
) -> PolarsResult<(DataFrame, usize)> {
    let original_height =
        df.height();

    let cleaned =
        df.unique_stable(
            None,
            UniqueKeepStrategy::First,
            None,
        )?;

    let affected_rows =
        original_height
            .saturating_sub(
                cleaned.height(),
            );

    Ok((
        cleaned,
        affected_rows,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn duplicated_dataframe() -> DataFrame {
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
                    "region".into(),
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
        )
    }

    #[test]
    fn detects_duplicate_rows() {
        let df =
            duplicated_dataframe();

        let suggestion =
            analyze_duplicates(&df)
                .expect(
                    "Duplicate analysis should work",
                )
                .expect(
                    "Duplicate suggestion should exist",
                );

        assert_eq!(
            suggestion.kind,
            CleaningKind::RemoveDuplicateRows
        );

        assert_eq!(
            suggestion.affected_rows,
            2
        );

        assert_eq!(
            suggestion.confidence,
            1.0
        );
    }

    #[test]
    fn ignores_dataframe_without_duplicates() {
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

        let suggestion =
            analyze_duplicates(&df)
                .expect(
                    "Duplicate analysis should work",
                );

        assert!(
            suggestion.is_none()
        );
    }

    #[test]
    fn removes_duplicate_rows() {
        let df =
            duplicated_dataframe();

        let (
            cleaned,
            affected_rows,
        ) =
            apply_remove_duplicates(
                &df,
            )
            .expect(
                "Duplicate removal should work",
            );

        assert_eq!(
            affected_rows,
            2
        );

        assert_eq!(
            cleaned.height(),
            3
        );

        assert_eq!(
            cleaned.width(),
            2
        );
    }

    #[test]
    fn keeps_original_row_order() {
        let df =
            duplicated_dataframe();

        let (
            cleaned,
            _,
        ) =
            apply_remove_duplicates(
                &df,
            )
            .expect(
                "Duplicate removal should work",
            );

        let ids =
            cleaned
                .column("id")
                .unwrap()
                .as_materialized_series()
                .i64()
                .unwrap();

        assert_eq!(
            ids.get(0),
            Some(1)
        );

        assert_eq!(
            ids.get(1),
            Some(2)
        );

        assert_eq!(
            ids.get(2),
            Some(3)
        );
    }
}