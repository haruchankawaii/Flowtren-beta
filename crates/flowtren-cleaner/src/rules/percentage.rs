use polars::prelude::*;

use crate::{
    rules::numbers::parse_localized_number,
    suggestion::{
        CleaningKind,
        CleaningSuggestion,
    },
};

const MIN_PERCENTAGE_CONFIDENCE: f32 = 0.80;

pub fn analyze_percentage(
    series: &Series,
) -> Option<CleaningSuggestion> {
    if series.dtype() != &DataType::String {
        return None;
    }

    let Ok(strings) = series.str() else {
        return None;
    };

    let mut candidate_count = 0usize;
    let mut percentage_count = 0usize;
    let mut normalization_count = 0usize;

    for index in 0..strings.len() {
        let Some(value) = strings.get(index) else {
            continue;
        };

        let trimmed = value.trim();

        if trimmed.is_empty() {
            continue;
        }

        candidate_count += 1;

        let Some(number_part) =
            extract_percentage_number(trimmed)
        else {
            continue;
        };

        let Some(number) =
            parse_localized_number(number_part)
        else {
            continue;
        };

        percentage_count += 1;

        let normalized =
            format_number(number);

        if normalized != trimmed {
            normalization_count += 1;
        }
    }

    if candidate_count == 0
        || percentage_count == 0
        || normalization_count == 0
    {
        return None;
    }

    let confidence =
        percentage_count as f32
            / candidate_count as f32;

    if confidence < MIN_PERCENTAGE_CONFIDENCE {
        return None;
    }

    Some(
        CleaningSuggestion::new(
            series.name().to_string(),
            CleaningKind::NormalizePercentage,
            normalization_count,
            confidence,
        ),
    )
}

pub fn apply_percentage_normalization(
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
                let Some(value) =
                    strings.get(index)
                else {
                    return None;
                };

                let trimmed =
                    value.trim();

                let Some(number_part) =
                    extract_percentage_number(
                        trimmed,
                    )
                else {
                    return Some(
                        value.to_string(),
                    );
                };

                let Some(number) =
                    parse_localized_number(
                        number_part,
                    )
                else {
                    return Some(
                        value.to_string(),
                    );
                };

                let normalized =
                    format_number(number);

                if normalized != trimmed {
                    affected_rows += 1;
                }

                Some(normalized)
            })
            .collect();

    let column = Column::new(
        series.name().clone(),
        values,
    );

    Ok((
        column,
        affected_rows,
    ))
}

fn extract_percentage_number(
    value: &str,
) -> Option<&str> {
    let value = value.trim();

    if !value.ends_with('%') {
        return None;
    }

    let number_part =
        value
            .trim_end_matches('%')
            .trim();

    if number_part.is_empty() {
        return None;
    }

    Some(number_part)
}

fn format_number(
    number: f64,
) -> String {
    if number.fract() == 0.0 {
        return format!(
            "{:.0}",
            number
        );
    }

    let formatted =
        format!("{number:.10}");

    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_percentage_column() {
        let series =
            Series::new(
                "discount".into(),
                [
                    "10%",
                    "15%",
                    "5%",
                    "20%",
                ],
            );

        let suggestion =
            analyze_percentage(
                &series,
            )
            .expect(
                "Percentage should be detected",
            );

        assert_eq!(
            suggestion.kind,
            CleaningKind::NormalizePercentage
        );

        assert_eq!(
            suggestion.affected_rows,
            4
        );

        assert_eq!(
            suggestion.confidence,
            1.0
        );
    }

    #[test]
    fn detects_localized_percentage() {
        let series =
            Series::new(
                "rate".into(),
                [
                    "12,5%",
                    "25,75%",
                    "1.500%",
                ],
            );

        let suggestion =
            analyze_percentage(
                &series,
            )
            .expect(
                "Localized percentage should be detected",
            );

        assert_eq!(
            suggestion.affected_rows,
            3
        );

        assert_eq!(
            suggestion.confidence,
            1.0
        );
    }

    #[test]
    fn ignores_non_percentage_column() {
        let series =
            Series::new(
                "amount".into(),
                [
                    "10",
                    "20",
                    "30",
                ],
            );

        assert!(
            analyze_percentage(
                &series,
            )
            .is_none()
        );
    }

    #[test]
    fn applies_percentage_normalization() {
        let series =
            Series::new(
                "discount".into(),
                [
                    Some("10%"),
                    Some("12,5%"),
                    Some("1.500%"),
                    None,
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_percentage_normalization(
                &series,
            )
            .expect(
                "Percentage normalization should work",
            );

        assert_eq!(
            affected_rows,
            3
        );

        let strings =
            column
                .as_materialized_series()
                .str()
                .unwrap();

        assert_eq!(
            strings.get(0),
            Some("10")
        );

        assert_eq!(
            strings.get(1),
            Some("12.5")
        );

        assert_eq!(
            strings.get(2),
            Some("1500")
        );

        assert_eq!(
            strings.get(3),
            None
        );
    }

    #[test]
    fn preserves_invalid_percentage() {
        let series =
            Series::new(
                "discount".into(),
                [
                    "abc%",
                    "10%",
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_percentage_normalization(
                &series,
            )
            .expect(
                "Percentage normalization should work",
            );

        assert_eq!(
            affected_rows,
            1
        );

        let strings =
            column
                .as_materialized_series()
                .str()
                .unwrap();

        assert_eq!(
            strings.get(0),
            Some("abc%")
        );

        assert_eq!(
            strings.get(1),
            Some("10")
        );
    }
}