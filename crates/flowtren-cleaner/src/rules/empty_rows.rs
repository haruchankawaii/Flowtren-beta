use polars::prelude::*;

use crate::suggestion::{
    CleaningKind,
    CleaningSuggestion,
};

pub fn analyze_empty_rows(
    df: &DataFrame,
) -> Option<CleaningSuggestion> {
    if df.height() == 0 {
        return None;
    }

    let empty_rows =
        count_empty_rows(df);

    if empty_rows == 0 {
        return None;
    }

    Some(
        CleaningSuggestion::new(
            "__row__",
            CleaningKind::RemoveEmptyRows,
            empty_rows,
            1.0,
        ),
    )
}

pub fn apply_remove_empty_rows(
    df: &DataFrame,
) -> PolarsResult<(DataFrame, usize)> {
    if df.height() == 0 {
        return Ok((
            df.clone(),
            0,
        ));
    }

    let keep_mask: Vec<bool> =
        (0..df.height())
            .map(|row_index| {
                !is_row_empty(
                    df,
                    row_index,
                )
            })
            .collect();

    let mask = BooleanChunked::from_slice(
        "keep".into(),
        &keep_mask,
    );

    let cleaned =
        df.filter(&mask)?;

    let affected_rows =
        df.height()
            .saturating_sub(
                cleaned.height(),
            );

    Ok((
        cleaned,
        affected_rows,
    ))
}

fn count_empty_rows(
    df: &DataFrame,
) -> usize {
    (0..df.height())
        .filter(|row_index| {
            is_row_empty(
                df,
                *row_index,
            )
        })
        .count()
}

fn is_row_empty(
    df: &DataFrame,
    row_index: usize,
) -> bool {
    df.columns()
        .iter()
        .all(|column| {
            let series =
                column.as_materialized_series();

            if series
                .is_null()
                .get(row_index)
                .unwrap_or(false)
            {
                return true;
            }

            if series.dtype()
                == &DataType::String
            {
                let Ok(strings) =
                    series.str()
                else {
                    return false;
                };

                return strings
                    .get(row_index)
                    .map(|value| {
                        value
                            .trim()
                            .is_empty()
                    })
                    .unwrap_or(true);
            }

            false
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_empty_rows() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "name".into(),
                        vec![
                            Some("Alice"),
                            Some(""),
                            Some("Bob"),
                        ],
                    ),
                    Column::new(
                        "city".into(),
                        vec![
                            Some("Jakarta"),
                            Some("   "),
                            Some("Bandung"),
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let suggestion =
            analyze_empty_rows(&df)
                .expect(
                    "Empty row should be detected",
                );

        assert_eq!(
            suggestion.kind,
            CleaningKind::RemoveEmptyRows
        );

        assert_eq!(
            suggestion.affected_rows,
            1
        );
    }

    #[test]
    fn removes_empty_rows() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "name".into(),
                        vec![
                            Some("Alice"),
                            Some(""),
                            Some("Bob"),
                        ],
                    ),
                    Column::new(
                        "city".into(),
                        vec![
                            Some("Jakarta"),
                            Some(" "),
                            Some("Bandung"),
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let (
            cleaned,
            affected_rows,
        ) =
            apply_remove_empty_rows(
                &df,
            )
            .expect(
                "Empty row removal should work",
            );

        assert_eq!(
            affected_rows,
            1
        );

        assert_eq!(
            cleaned.height(),
            2
        );
    }

    #[test]
    fn does_not_remove_partially_filled_row() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "name".into(),
                        vec![
                            Some("Alice"),
                            Some(""),
                        ],
                    ),
                    Column::new(
                        "city".into(),
                        vec![
                            Some(""),
                            Some("Bandung"),
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let suggestion =
            analyze_empty_rows(&df);

        assert!(
            suggestion.is_none()
        );
    }
}