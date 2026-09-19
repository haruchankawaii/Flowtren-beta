use polars::prelude::*;

use crate::suggestion::{
    CleaningKind,
    CleaningSuggestion,
};

const MIN_NUMERIC_CONFIDENCE: f32 = 0.80;

pub fn analyze_number_format(
    series: &Series,
) -> Option<CleaningSuggestion> {
    if series.dtype() != &DataType::String {
        return None;
    }

    let Ok(strings) = series.str() else {
        return None;
    };

    let mut candidate_count = 0usize;
    let mut numeric_count = 0usize;
    let mut normalization_count = 0usize;

    for index in 0..strings.len() {
        let Some(value) = strings.get(index) else {
            continue;
        };

        let trimmed = value.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Jangan anggap currency / percentage sebagai angka biasa.
        // Itu akan ditangani rule khusus nanti.
        if contains_currency_marker(trimmed)
            || trimmed.ends_with('%')
        {
            continue;
        }

        // Jangan mencoba nilai dengan huruf.
        // Contoh:
        // CUST-001
        // INV2026
        //
        // Itu lebih mungkin identifier.
        if trimmed
            .chars()
            .any(|c| c.is_ascii_alphabetic())
        {
            continue;
        }

        candidate_count += 1;

        let Some(number) =
            parse_localized_number(trimmed)
        else {
            continue;
        };

        numeric_count += 1;

        let normalized =
            format_number(number);

        if normalized != trimmed {
            normalization_count += 1;
        }
    }

    if candidate_count == 0
        || numeric_count == 0
        || normalization_count == 0
    {
        return None;
    }

    let confidence =
        numeric_count as f32
            / candidate_count as f32;

    if confidence < MIN_NUMERIC_CONFIDENCE {
        return None;
    }

    Some(
        CleaningSuggestion::new(
            series.name().to_string(),
            CleaningKind::NormalizeNumberFormat,
            normalization_count,
            confidence,
        ),
    )
}

pub fn apply_number_format(
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

                if trimmed.is_empty()
                    || contains_currency_marker(trimmed)
                    || trimmed.ends_with('%')
                    || trimmed
                        .chars()
                        .any(|c| c.is_ascii_alphabetic())
                {
                    return Some(
                        value.to_string()
                    );
                }

                let Some(number) =
                    parse_localized_number(trimmed)
                else {
                    // Tidak berhasil diparse:
                    // pertahankan nilai original.
                    return Some(
                        value.to_string()
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

pub fn parse_localized_number(
    value: &str,
) -> Option<f64> {
    let value = value.trim();

    if value.is_empty() {
        return None;
    }

    // Jangan menerima huruf, currency, atau percentage.
    if value
        .chars()
        .any(|c| {
            c.is_ascii_alphabetic()
                || matches!(
                    c,
                    '%' | '$' | '€' | '£'
                )
        })
    {
        return None;
    }

    // Hanya karakter yang masuk akal
    // sebagai representasi angka.
    if !value
        .chars()
        .all(|c| {
            c.is_ascii_digit()
                || matches!(
                    c,
                    '.' | ',' | '-' | '+' | ' '
                )
        })
    {
        return None;
    }

    let compact =
        value.replace(' ', "");

    if compact.is_empty() {
        return None;
    }

    let has_dot =
        compact.contains('.');

    let has_comma =
        compact.contains(',');

    // -----------------------------------
    // Ada titik + koma.
    //
    // 1.500.000,50
    // 1,500,000.50
    //
    // Separator paling kanan dianggap
    // sebagai decimal separator.
    // -----------------------------------

    if has_dot && has_comma {
        let last_dot =
            compact.rfind('.')?;

        let last_comma =
            compact.rfind(',')?;

        let normalized =
            if last_comma > last_dot {
                // Indonesia:
                // 1.500.000,50
                compact
                    .replace('.', "")
                    .replace(',', ".")
            } else {
                // US:
                // 1,500,000.50
                compact
                    .replace(',', "")
            };

        return normalized
            .parse::<f64>()
            .ok();
    }

    // -----------------------------------
    // Hanya koma.
    // -----------------------------------

    if has_comma {
        return parse_single_separator(
            &compact,
            ',',
        );
    }

    // -----------------------------------
    // Hanya titik.
    //
    // Penting:
    // kita cek heuristic dulu supaya:
    //
    // 1.500 -> 1500
    // 12.50 -> 12.5
    // -----------------------------------

    if has_dot {
        return parse_single_separator(
            &compact,
            '.',
        );
    }

    // -----------------------------------
    // Tidak ada separator.
    //
    // 1000
    // -1000
    // +1000
    // -----------------------------------

    compact
        .parse::<f64>()
        .ok()
}

fn parse_single_separator(
    value: &str,
    separator: char,
) -> Option<f64> {
    let separator_count =
        value.matches(separator).count();

    if separator_count == 0 {
        return value
            .parse::<f64>()
            .ok();
    }

    // Banyak separator:
    //
    // 1.500.000
    // 1,500,000
    //
    // semuanya dianggap thousand separator
    // jika setiap grup setelah grup pertama
    // memiliki 3 digit.
    if separator_count > 1 {
        let unsigned =
            value
                .trim_start_matches(['-', '+']);

        let parts: Vec<&str> =
            unsigned
                .split(separator)
                .collect();

        if parts.len() < 2 {
            return None;
        }

        let valid_groups =
            parts
                .iter()
                .skip(1)
                .all(|part| {
                    part.len() == 3
                        && part
                            .chars()
                            .all(|c| {
                                c.is_ascii_digit()
                            })
                });

        if !valid_groups {
            return None;
        }

        let normalized =
            value.replace(separator, "");

        return normalized
            .parse::<f64>()
            .ok();
    }

    let separator_index =
        value.rfind(separator)?;

    let fractional_digits =
        value.len()
            .saturating_sub(
                separator_index + 1,
            );

    // 1.500 / 1,500
    //
    // Kita interpretasi sebagai 1500.
    if fractional_digits == 3 {
        let normalized =
            value.replace(separator, "");

        return normalized
            .parse::<f64>()
            .ok();
    }

    // 12.50 / 12,50
    //
    // dianggap decimal.
    if fractional_digits == 1
        || fractional_digits == 2
    {
        let normalized =
            if separator == ',' {
                value.replace(',', ".")
            } else {
                value.to_string()
            };

        return normalized
            .parse::<f64>()
            .ok();
    }

    // Contoh 1000,0000 terlalu ambigu
    // untuk kita normalisasi otomatis.
    None
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

fn contains_currency_marker(
    value: &str,
) -> bool {
    let lowercase =
        value.to_ascii_lowercase();

    lowercase.starts_with("rp")
        || lowercase.starts_with("idr")
        || value.starts_with('$')
        || value.starts_with('€')
        || value.starts_with('£')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_integer() {
        assert_eq!(
            parse_localized_number(
                "1500"
            ),
            Some(1500.0)
        );
    }

    #[test]
    fn parses_indonesian_thousands() {
        assert_eq!(
            parse_localized_number(
                "1.500.000"
            ),
            Some(1_500_000.0)
        );
    }

    #[test]
    fn parses_us_thousands() {
        assert_eq!(
            parse_localized_number(
                "1,500,000"
            ),
            Some(1_500_000.0)
        );
    }

    #[test]
    fn parses_indonesian_decimal() {
        assert_eq!(
            parse_localized_number(
                "1.500,50"
            ),
            Some(1500.50)
        );
    }

    #[test]
    fn parses_us_decimal() {
        assert_eq!(
            parse_localized_number(
                "1,500.50"
            ),
            Some(1500.50)
        );
    }

    #[test]
    fn rejects_identifiers() {
        assert_eq!(
            parse_localized_number(
                "CUST-001"
            ),
            None
        );
    }

    #[test]
    fn rejects_percentage() {
        assert_eq!(
            parse_localized_number(
                "10%"
            ),
            None
        );
    }

    #[test]
    fn rejects_currency() {
        assert_eq!(
            parse_localized_number(
                "Rp 1.500.000"
            ),
            None
        );
    }

    #[test]
    fn detects_number_normalization() {
        let series =
            Series::new(
                "amount".into(),
                [
                    "1.500",
                    "2.500",
                    "3.000",
                    "4000",
                ],
            );

        let suggestion =
            analyze_number_format(
                &series,
            )
            .expect(
                "Number normalization should be detected",
            );

        assert_eq!(
            suggestion.kind,
            CleaningKind::NormalizeNumberFormat
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
    fn applies_number_normalization() {
        let series =
            Series::new(
                "amount".into(),
                [
                    Some("1.500"),
                    Some("2,500"),
                    Some("12,50"),
                    Some("hello"),
                    None,
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_number_format(
                &series,
            )
            .expect(
                "Number normalization should work",
            );

        assert_eq!(
            affected_rows,
            3
        );

        let cleaned =
            column
                .as_materialized_series();

        let strings =
            cleaned
                .str()
                .unwrap();

        assert_eq!(
            strings.get(0),
            Some("1500")
        );

        assert_eq!(
            strings.get(1),
            Some("2500")
        );

        assert_eq!(
            strings.get(2),
            Some("12.5")
        );

        // Nilai non-numeric tidak dihapus.
        assert_eq!(
            strings.get(3),
            Some("hello")
        );

        assert_eq!(
            strings.get(4),
            None
        );
    }
}