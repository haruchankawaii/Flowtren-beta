use polars::prelude::*;

use crate::{
    rules::numbers::parse_localized_number,
    suggestion::{
        CleaningKind,
        CleaningSuggestion,
    },
};

const MIN_CURRENCY_CONFIDENCE: f32 = 0.80;

pub fn analyze_currency(
    series: &Series,
) -> Option<CleaningSuggestion> {
    if series.dtype() != &DataType::String {
        return None;
    }

    let Ok(strings) = series.str() else {
        return None;
    };

    let mut candidate_count = 0usize;
    let mut currency_count = 0usize;
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
            extract_currency_number(trimmed)
        else {
            continue;
        };

        let Some(number) =
            parse_localized_number(number_part)
        else {
            continue;
        };

        currency_count += 1;

        let normalized =
            format_number(number);

        if normalized != trimmed {
            normalization_count += 1;
        }
    }

    if candidate_count == 0
        || currency_count == 0
        || normalization_count == 0
    {
        return None;
    }

    let confidence =
        currency_count as f32
            / candidate_count as f32;

    if confidence < MIN_CURRENCY_CONFIDENCE {
        return None;
    }

    Some(
        CleaningSuggestion::new(
            series.name().to_string(),
            CleaningKind::NormalizeCurrency,
            normalization_count,
            confidence,
        ),
    )
}

pub fn apply_currency_normalization(
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

                let trimmed = value.trim();

                let Some(number_part) =
                    extract_currency_number(trimmed)
                else {
                    return Some(
                        value.to_string(),
                    );
                };

                let Some(number) =
                    parse_localized_number(number_part)
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

fn extract_currency_number(
    value: &str,
) -> Option<&str> {
    let value = value.trim();

    if let Some(rest) =
        value.strip_prefix("Rp")
    {
        return Some(rest.trim());
    }

    if let Some(rest) =
        value.strip_prefix("rp")
    {
        return Some(rest.trim());
    }

    if let Some(rest) =
        value.strip_prefix("IDR")
    {
        return Some(rest.trim());
    }

    if let Some(rest) =
        value.strip_prefix("idr")
    {
        return Some(rest.trim());
    }

    if let Some(rest) =
        value.strip_prefix('$')
    {
        return Some(rest.trim());
    }

    if let Some(rest) =
        value.strip_prefix('€')
    {
        return Some(rest.trim());
    }

    if let Some(rest) =
        value.strip_prefix('£')
    {
        return Some(rest.trim());
    }

    None
}

fn format_number(
    number: f64,
) -> String {
    if number.fract() == 0.0 {
        return format!("{:.0}", number);
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
    fn detects_currency_column() {
        let series =
            Series::new(
                "price".into(),
                [
                    "Rp 1.500.000",
                    "Rp 2.000.000",
                    "IDR 500000",
                    "Rp 750.000",
                ],
            );

        let suggestion =
            analyze_currency(
                &series,
            )
            .expect(
                "Currency should be detected",
            );

        assert_eq!(
            suggestion.kind,
            CleaningKind::NormalizeCurrency
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
    fn ignores_non_currency_column() {
        let series =
            Series::new(
                "name".into(),
                [
                    "Jakarta",
                    "Bandung",
                    "Surabaya",
                ],
            );

        assert!(
            analyze_currency(&series)
                .is_none()
        );
    }

    #[test]
    fn applies_rupiah_normalization() {
        let series =
            Series::new(
                "price".into(),
                [
                    Some("Rp 1.500.000"),
                    Some("IDR 250000"),
                    Some("Rp 12.500,50"),
                    None,
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_currency_normalization(
                &series,
            )
            .expect(
                "Currency normalization should work",
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
            Some("1500000")
        );

        assert_eq!(
            strings.get(1),
            Some("250000")
        );

        assert_eq!(
            strings.get(2),
            Some("12500.5")
        );

        assert_eq!(
            strings.get(3),
            None
        );
    }

    #[test]
    fn applies_international_currency() {
        let series =
            Series::new(
                "price".into(),
                [
                    "$1,500.50",
                    "€ 99,90",
                    "£1000",
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_currency_normalization(
                &series,
            )
            .expect(
                "Currency normalization should work",
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
            Some("1500.5")
        );

        assert_eq!(
            strings.get(1),
            Some("99.9")
        );

        assert_eq!(
            strings.get(2),
            Some("1000")
        );
    }

    #[test]
    fn preserves_invalid_currency_value() {
        let series =
            Series::new(
                "price".into(),
                [
                    "Rp abc",
                    "Rp 1500",
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_currency_normalization(
                &series,
            )
            .expect(
                "Currency normalization should work",
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
            Some("Rp abc")
        );

        assert_eq!(
            strings.get(1),
            Some("1500")
        );
    }
}