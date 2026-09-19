use polars::prelude::*;

use crate::suggestion::{
    CleaningKind,
    CleaningSuggestion,
};

pub fn analyze_null_like_values(
    series: &Series,
) -> Vec<CleaningSuggestion> {
    let mut suggestions =
        Vec::new();

    if series.dtype() != &DataType::String {
        return suggestions;
    }

    let Ok(strings) = series.str() else {
        return suggestions;
    };

    let mut empty_count = 0usize;
    let mut marker_count = 0usize;

    for index in 0..strings.len() {
        let Some(value) =
            strings.get(index)
        else {
            continue;
        };

        let trimmed = value.trim();

        if trimmed.is_empty() {
            empty_count += 1;
            continue;
        }

        if is_missing_marker(trimmed) {
            marker_count += 1;
        }
    }

    let column_name =
        series.name().to_string();

    if empty_count > 0 {
        suggestions.push(
            CleaningSuggestion::new(
                column_name.clone(),
                CleaningKind::EmptyStringToNull,
                empty_count,
                1.0,
            ),
        );
    }

    if marker_count > 0 {
        suggestions.push(
            CleaningSuggestion::new(
                column_name,
                CleaningKind::NormalizeMissingMarker,
                marker_count,
                1.0,
            ),
        );
    }

    suggestions
}

pub fn apply_empty_string_to_null(
    series: &Series,
) -> PolarsResult<(Column, usize)> {
    if series.dtype() != &DataType::String {
        return Ok((
            series.clone().into_column(),
            0,
        ));
    }

    let strings = series.str()?;

    let mut affected_rows = 0usize;

    let values: Vec<Option<String>> =
        (0..strings.len())
            .map(|index| {
                let value = strings.get(index)?;

                if value.trim().is_empty() {
                    affected_rows += 1;
                    None
                } else {
                    Some(value.to_string())
                }
            })
            .collect();

    let column = Column::new(
        series.name().clone(),
        values,
    );

    Ok((column, affected_rows))
}

pub fn apply_missing_markers_to_null(
    series: &Series,
) -> PolarsResult<(Column, usize)> {
    if series.dtype() != &DataType::String {
        return Ok((
            series.clone().into_column(),
            0,
        ));
    }

    let strings = series.str()?;

    let mut affected_rows = 0usize;

    let values: Vec<Option<String>> =
        (0..strings.len())
            .map(|index| {
                let value = strings.get(index)?;

                if is_missing_marker(
                    value.trim(),
                ) {
                    affected_rows += 1;
                    None
                } else {
                    Some(value.to_string())
                }
            })
            .collect();

    let column = Column::new(
        series.name().clone(),
        values,
    );

    Ok((column, affected_rows))
}

fn is_missing_marker(
    value: &str,
) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "na"
            | "n/a"
            | "null"
            | "none"
            | "nan"
            | "missing"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_empty_string() {
        let series = Series::new(
            "region".into(),
            [
                "Jakarta",
                "",
                "Bandung",
            ],
        );

        let suggestions =
            analyze_null_like_values(
                &series,
            );

        let suggestion =
            suggestions
                .iter()
                .find(|suggestion| {
                    suggestion.kind
                        == CleaningKind::EmptyStringToNull
                })
                .expect(
                    "Empty string should be detected",
                );

        assert_eq!(
            suggestion.affected_rows,
            1
        );
    }

    #[test]
    fn detects_missing_markers() {
        let series = Series::new(
            "region".into(),
            [
                "Jakarta",
                "N/A",
                "NULL",
                "missing",
            ],
        );

        let suggestions =
            analyze_null_like_values(
                &series,
            );

        let suggestion =
            suggestions
                .iter()
                .find(|suggestion| {
                    suggestion.kind
                        == CleaningKind::NormalizeMissingMarker
                })
                .expect(
                    "Missing markers should be detected",
                );

        assert_eq!(
            suggestion.affected_rows,
            3
        );
    }

    #[test]
    fn converts_empty_strings_to_null() {
        let series = Series::new(
            "region".into(),
            [
                Some("Jakarta"),
                Some(""),
                Some("   "),
                None,
            ],
        );

        let (column, affected_rows) =
            apply_empty_string_to_null(
                &series,
            )
            .expect(
                "Empty string cleaning should work",
            );

        assert_eq!(
            affected_rows,
            2
        );

        let cleaned =
            column.as_materialized_series();

        let strings =
            cleaned.str().unwrap();

        assert_eq!(
            strings.get(0),
            Some("Jakarta")
        );

        assert_eq!(
            strings.get(1),
            None
        );

        assert_eq!(
            strings.get(2),
            None
        );

        assert_eq!(
            strings.get(3),
            None
        );
    }

    #[test]
    fn converts_missing_markers_to_null() {
        let series = Series::new(
            "region".into(),
            [
                Some("Jakarta"),
                Some("N/A"),
                Some("NULL"),
                Some("missing"),
                None,
            ],
        );

        let (column, affected_rows) =
            apply_missing_markers_to_null(
                &series,
            )
            .expect(
                "Missing marker cleaning should work",
            );

        assert_eq!(
            affected_rows,
            3
        );

        let cleaned =
            column.as_materialized_series();

        let strings =
            cleaned.str().unwrap();

        assert_eq!(
            strings.get(0),
            Some("Jakarta")
        );

        assert_eq!(
            strings.get(1),
            None
        );

        assert_eq!(
            strings.get(2),
            None
        );

        assert_eq!(
            strings.get(3),
            None
        );

        assert_eq!(
            strings.get(4),
            None
        );
    }
}