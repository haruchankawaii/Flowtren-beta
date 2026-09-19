use polars::prelude::*;

use crate::{
    audit::CleaningAudit,
    rules::{
        casing::{
            analyze_casing,
            apply_casing_normalization,
        },
        categories::{
            analyze_categories,
            apply_category_normalization,
        },
        currency::{
            analyze_currency,
            apply_currency_normalization,
        },
        dates::{
            analyze_dates,
            apply_date_normalization,
        },
        duplicates::{
            analyze_duplicates,
            apply_remove_duplicates,
        },
        empty_rows::{
            analyze_empty_rows,
            apply_remove_empty_rows,
        },
        nulls::{
            analyze_null_like_values,
            apply_empty_string_to_null,
            apply_missing_markers_to_null,
        },
        numbers::{
            analyze_number_format,
            apply_number_format,
        },
        percentage::{
            analyze_percentage,
            apply_percentage_normalization,
        },
        whitespace::{
            analyze_whitespace,
            apply_trim_whitespace,
        },
    },
    suggestion::{
        CleaningKind,
        CleaningSuggestion,
    },
};

#[derive(Debug, Clone)]
pub struct CleaningResult {
    pub dataframe: DataFrame,
    pub audit: CleaningAudit,
}

pub fn analyze_dataframe(
    df: &DataFrame,
) -> PolarsResult<Vec<CleaningSuggestion>> {
    let mut suggestions =
        Vec::new();

    // -----------------------------------------
    // Column-level analysis
    // -----------------------------------------
    for column in df.columns() {
        let series =
            column.as_materialized_series();

        if let Some(suggestion) =
            analyze_whitespace(series)
        {
            suggestions.push(
                suggestion,
            );
        }

        suggestions.extend(
            analyze_null_like_values(
                series,
            ),
        );

        if let Some(suggestion) =
            analyze_currency(series)
        {
            suggestions.push(
                suggestion,
            );
        }

        if let Some(suggestion) =
            analyze_percentage(series)
        {
            suggestions.push(
                suggestion,
            );
        }

        if let Some(suggestion) =
            analyze_dates(series)
        {
            suggestions.push(
                suggestion,
            );
        }

        if let Some(suggestion) =
            analyze_number_format(series)
        {
            suggestions.push(
                suggestion,
            );
        }

        if let Some(suggestion) =
            analyze_categories(series)
        {
            suggestions.push(
                suggestion,
            );
        }

        if let Some(suggestion) =
            analyze_casing(series)
        {
            suggestions.push(
                suggestion,
            );
        }
    }

    // -----------------------------------------
    // Row-level analysis
    // -----------------------------------------
    if let Some(suggestion) =
        analyze_empty_rows(df)
    {
        suggestions.push(
            suggestion,
        );
    }

    if let Some(suggestion) =
        analyze_duplicates(df)?
    {
        suggestions.push(
            suggestion,
        );
    }

    Ok(suggestions)
}

pub fn apply_suggestions(
    df: &DataFrame,
    suggestions: &[CleaningSuggestion],
) -> PolarsResult<CleaningResult> {
    let mut cleaned =
        df.clone();

    let mut audit =
        CleaningAudit::new();

    // -----------------------------------------
    // Row-level rules first
    // -----------------------------------------

    for suggestion in suggestions {
        if suggestion.kind
            == CleaningKind::RemoveEmptyRows
        {
            apply_single_suggestion(
                &mut cleaned,
                suggestion,
                &mut audit,
            )?;
        }
    }

    for suggestion in suggestions {
        if suggestion.kind
            == CleaningKind::RemoveDuplicateRows
        {
            apply_single_suggestion(
                &mut cleaned,
                suggestion,
                &mut audit,
            )?;
        }
    }

    // -----------------------------------------
    // Column-level rules
    // -----------------------------------------
    //
    // Categories dijalankan sebelum casing.
    //
    // Ini penting supaya variasi:
    //
    // Jakarta
    // jakarta
    //
    // diselesaikan oleh category normalization,
    // bukan oleh general casing rule.
    // -----------------------------------------

    const COLUMN_ORDER:
        &[CleaningKind] = &[
        CleaningKind::TrimWhitespace,
        CleaningKind::EmptyStringToNull,
        CleaningKind::NormalizeMissingMarker,
        CleaningKind::NormalizeCurrency,
        CleaningKind::NormalizePercentage,
        CleaningKind::NormalizeDate,
        CleaningKind::NormalizeNumberFormat,
        CleaningKind::NormalizeCategories,
        CleaningKind::NormalizeCasing,
    ];

    for kind in COLUMN_ORDER {
        for suggestion in suggestions {
            if &suggestion.kind
                == kind
            {
                apply_single_suggestion(
                    &mut cleaned,
                    suggestion,
                    &mut audit,
                )?;
            }
        }
    }

    Ok(
        CleaningResult {
            dataframe: cleaned,
            audit,
        },
    )
}

fn apply_single_suggestion(
    df: &mut DataFrame,
    suggestion: &CleaningSuggestion,
    audit: &mut CleaningAudit,
) -> PolarsResult<()> {
    // -----------------------------------------
    // Row-level rules
    // -----------------------------------------
    match suggestion.kind {
        CleaningKind::RemoveEmptyRows => {
            let (
                cleaned_df,
                affected_rows,
            ) =
                apply_remove_empty_rows(
                    df,
                )?;

            *df =
                cleaned_df;

            audit.record(
                "__row__",
                CleaningKind::RemoveEmptyRows,
                affected_rows,
            );

            return Ok(());
        }

        CleaningKind::RemoveDuplicateRows => {
            let (
                cleaned_df,
                affected_rows,
            ) =
                apply_remove_duplicates(
                    df,
                )?;

            *df =
                cleaned_df;

            audit.record(
                "__row__",
                CleaningKind::RemoveDuplicateRows,
                affected_rows,
            );

            return Ok(());
        }

        CleaningKind::TrimWhitespace
        | CleaningKind::EmptyStringToNull
        | CleaningKind::NormalizeMissingMarker
        | CleaningKind::NormalizeCurrency
        | CleaningKind::NormalizePercentage
        | CleaningKind::NormalizeDate
        | CleaningKind::NormalizeNumberFormat
        | CleaningKind::NormalizeCategories
        | CleaningKind::NormalizeCasing => {}
    }

    // -----------------------------------------
    // Column-level rules
    // -----------------------------------------
    let column =
        df.column(
            &suggestion.column,
        )?;

    let series =
        column.as_materialized_series();

    let (
        cleaned_column,
        affected_rows,
    ) = match suggestion.kind {
        CleaningKind::TrimWhitespace => {
            apply_trim_whitespace(
                series,
            )?
        }

        CleaningKind::EmptyStringToNull => {
            apply_empty_string_to_null(
                series,
            )?
        }

        CleaningKind::NormalizeMissingMarker => {
            apply_missing_markers_to_null(
                series,
            )?
        }

        CleaningKind::NormalizeCurrency => {
            apply_currency_normalization(
                series,
            )?
        }

        CleaningKind::NormalizePercentage => {
            apply_percentage_normalization(
                series,
            )?
        }

        CleaningKind::NormalizeDate => {
            apply_date_normalization(
                series,
            )?
        }

        CleaningKind::NormalizeNumberFormat => {
            apply_number_format(
                series,
            )?
        }

        CleaningKind::NormalizeCategories => {
            apply_category_normalization(
                series,
            )?
        }

        CleaningKind::NormalizeCasing => {
            apply_casing_normalization(
                series,
            )?
        }

        CleaningKind::RemoveEmptyRows
        | CleaningKind::RemoveDuplicateRows => {
            unreachable!()
        }
    };

    df.with_column(
        cleaned_column,
    )?;

    audit.record(
        suggestion.column.clone(),
        suggestion.kind.clone(),
        affected_rows,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_category_normalization() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            2_i64,
                            3_i64,
                            4_i64,
                        ],
                    ),
                    Column::new(
                        "city".into(),
                        vec![
                            "Jakarta",
                            "Jakarta",
                            "Jakarta",
                            "jakarta",
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let suggestions =
            analyze_dataframe(
                &df,
            )
            .expect(
                "Analysis should succeed",
            );

        let category =
            suggestions
                .iter()
                .find(
                    |suggestion| {
                        suggestion.kind
                            == CleaningKind::NormalizeCategories
                    },
                )
                .expect(
                    "Category suggestion should exist",
                );

        assert_eq!(
            category.affected_rows,
            1
        );

        assert!(
            category.confidence
                >= 0.75
        );

        // Tidak ada duplicate row karena
        // setiap id berbeda.
        assert!(
            !suggestions.iter().any(
                |suggestion| {
                    suggestion.kind
                        == CleaningKind::RemoveDuplicateRows
                }
            )
        );
    }

    #[test]
    fn applies_category_normalization() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            2_i64,
                            3_i64,
                            4_i64,
                        ],
                    ),
                    Column::new(
                        "city".into(),
                        vec![
                            "Jakarta",
                            "Jakarta",
                            "Jakarta",
                            "jakarta",
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let suggestions =
            analyze_dataframe(
                &df,
            )
            .expect(
                "Analysis should succeed",
            );

        // Pastikan fixture ini tidak
        // memicu duplicate removal.
        assert!(
            !suggestions.iter().any(
                |suggestion| {
                    suggestion.kind
                        == CleaningKind::RemoveDuplicateRows
                }
            )
        );

        assert!(
            suggestions.iter().any(
                |suggestion| {
                    suggestion.kind
                        == CleaningKind::NormalizeCategories
                }
            )
        );

        let result =
            apply_suggestions(
                &df,
                &suggestions,
            )
            .expect(
                "Cleaning should succeed",
            );

        // Semua row harus tetap ada.
        assert_eq!(
            result.dataframe.height(),
            4
        );

        let strings =
            result
                .dataframe
                .column("city")
                .unwrap()
                .as_materialized_series()
                .str()
                .unwrap();

        assert_eq!(
            strings.get(0),
            Some("Jakarta")
        );

        assert_eq!(
            strings.get(1),
            Some("Jakarta")
        );

        assert_eq!(
            strings.get(2),
            Some("Jakarta")
        );

        assert_eq!(
            strings.get(3),
            Some("Jakarta")
        );
    }

    #[test]
    fn detects_casing_normalization() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            2_i64,
                            3_i64,
                            4_i64,
                            5_i64,
                        ],
                    ),
                    Column::new(
                        "name".into(),
                        vec![
                            "Alice",
                            "Bob",
                            "Charlie",
                            "David",
                            "edward",
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let suggestions =
            analyze_dataframe(
                &df,
            )
            .expect(
                "Analysis should succeed",
            );

        let suggestion =
            suggestions
                .iter()
                .find(
                    |suggestion| {
                        suggestion.kind
                            == CleaningKind::NormalizeCasing
                    },
                )
                .expect(
                    "Casing suggestion should exist",
                );

        assert_eq!(
            suggestion.affected_rows,
            1
        );

        assert_eq!(
            suggestion.confidence,
            0.8
        );
    }

    #[test]
    fn applies_casing_normalization() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            2_i64,
                            3_i64,
                            4_i64,
                            5_i64,
                        ],
                    ),
                    Column::new(
                        "name".into(),
                        vec![
                            "Alice",
                            "Bob",
                            "Charlie",
                            "David",
                            "edward",
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let suggestions =
            analyze_dataframe(
                &df,
            )
            .expect(
                "Analysis should succeed",
            );

        assert!(
            !suggestions.iter().any(
                |suggestion| {
                    suggestion.kind
                        == CleaningKind::RemoveDuplicateRows
                }
            )
        );

        let result =
            apply_suggestions(
                &df,
                &suggestions,
            )
            .expect(
                "Cleaning should succeed",
            );

        assert_eq!(
            result.dataframe.height(),
            5
        );

        let strings =
            result
                .dataframe
                .column("name")
                .unwrap()
                .as_materialized_series()
                .str()
                .unwrap();

        assert_eq!(
            strings.get(0),
            Some("Alice")
        );

        assert_eq!(
            strings.get(4),
            Some("Edward")
        );
    }

    #[test]
    fn normalization_does_not_silently_remove_new_duplicates() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            1_i64,
                        ],
                    ),
                    Column::new(
                        "region".into(),
                        vec![
                            "",
                            "N/A",
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let suggestions =
            analyze_dataframe(
                &df,
            )
            .expect(
                "Analysis should succeed",
            );

        // Sebelum normalization:
        //
        // 1, ""
        // 1, "N/A"
        //
        // belum duplicate.
        assert!(
            !suggestions.iter().any(
                |suggestion| {
                    suggestion.kind
                        == CleaningKind::RemoveDuplicateRows
                }
            )
        );

        // Juga bukan empty row karena
        // kolom id masih terisi.
        assert!(
            !suggestions.iter().any(
                |suggestion| {
                    suggestion.kind
                        == CleaningKind::RemoveEmptyRows
                }
            )
        );

        let result =
            apply_suggestions(
                &df,
                &suggestions,
            )
            .expect(
                "Cleaning should succeed",
            );

        // Setelah normalization:
        //
        // 1, null
        // 1, null
        //
        // sekarang sama, tetapi Flowtren
        // tidak boleh membuat keputusan baru
        // yang tidak ada pada analysis snapshot.
        assert_eq!(
            result.dataframe.height(),
            2
        );

        let regions =
            result
                .dataframe
                .column("region")
                .unwrap()
                .as_materialized_series()
                .str()
                .unwrap();

        assert_eq!(
            regions.get(0),
            None
        );

        assert_eq!(
            regions.get(1),
            None
        );
    }

    #[test]
    fn original_dataframe_is_not_modified() {
        let df =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "id".into(),
                        vec![
                            1_i64,
                            2_i64,
                            3_i64,
                            4_i64,
                            5_i64,
                        ],
                    ),
                    Column::new(
                        "name".into(),
                        vec![
                            "Alice",
                            "Bob",
                            "Charlie",
                            "David",
                            "edward",
                        ],
                    ),
                ],
            )
            .expect(
                "DataFrame should build",
            );

        let suggestions =
            analyze_dataframe(
                &df,
            )
            .expect(
                "Analysis should succeed",
            );

        let _result =
            apply_suggestions(
                &df,
                &suggestions,
            )
            .expect(
                "Cleaning should succeed",
            );

        let original =
            df.column("name")
                .unwrap()
                .as_materialized_series()
                .str()
                .unwrap();

        assert_eq!(
            original.get(4),
            Some("edward")
        );
    }
}