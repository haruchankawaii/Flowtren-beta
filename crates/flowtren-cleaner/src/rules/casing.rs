use std::collections::{
    HashMap,
    HashSet,
};

use polars::prelude::*;

use crate::suggestion::{
    CleaningKind,
    CleaningSuggestion,
};

const MIN_CASING_CONFIDENCE: f32 = 0.80;
const MIN_CASING_VALUES: usize = 4;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
enum CaseStyle {
    Lower,
    Upper,
    Title,
}

pub fn analyze_casing(
    series: &Series,
) -> Option<CleaningSuggestion> {
    if series.dtype()
        != &DataType::String
    {
        return None;
    }

    // Kalau ada "Jakarta/jakarta",
    // biarkan categories.rs yang menangani.
    if has_case_variant_collisions(
        series,
    ) {
        return None;
    }

    let (
        dominant_style,
        confidence,
        affected_rows,
    ) =
        infer_dominant_style(
            series,
        )?;

    let _ =
        dominant_style;

    if confidence
        < MIN_CASING_CONFIDENCE
        || affected_rows == 0
    {
        return None;
    }

    Some(
        CleaningSuggestion::new(
            series.name().to_string(),
            CleaningKind::NormalizeCasing,
            affected_rows,
            confidence,
        ),
    )
}

pub fn apply_casing_normalization(
    series: &Series,
) -> PolarsResult<(Column, usize)> {
    if series.dtype()
        != &DataType::String
    {
        return Ok((
            series.clone().into_column(),
            0,
        ));
    }

    if has_case_variant_collisions(
        series,
    ) {
        return Ok((
            series.clone().into_column(),
            0,
        ));
    }

    let Some((
        dominant_style,
        confidence,
        _,
    )) =
        infer_dominant_style(
            series,
        )
    else {
        return Ok((
            series.clone().into_column(),
            0,
        ));
    };

    if confidence
        < MIN_CASING_CONFIDENCE
    {
        return Ok((
            series.clone().into_column(),
            0,
        ));
    }

    let strings =
        series.str()?;

    let mut affected_rows =
        0usize;

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

                if !is_casing_candidate(
                    trimmed,
                ) {
                    return Some(
                        value.to_string(),
                    );
                }

                let Some(style) =
                    detect_case_style(
                        trimmed,
                    )
                else {
                    return Some(
                        value.to_string(),
                    );
                };

                if style
                    == dominant_style
                {
                    return Some(
                        value.to_string(),
                    );
                }

                let normalized =
                    apply_case_style(
                        trimmed,
                        dominant_style,
                    );

                if normalized
                    != trimmed
                {
                    affected_rows += 1;
                }

                Some(normalized)
            })
            .collect();

    let column =
        Column::new(
            series.name().clone(),
            values,
        );

    Ok((
        column,
        affected_rows,
    ))
}

fn infer_dominant_style(
    series: &Series,
) -> Option<(
    CaseStyle,
    f32,
    usize,
)> {
    let strings =
        series.str().ok()?;

    let mut counts:
        HashMap<CaseStyle, usize> =
        HashMap::new();

    let mut candidate_count =
        0usize;

    for index in 0..strings.len() {
        let Some(value) =
            strings.get(index)
        else {
            continue;
        };

        let trimmed =
            value.trim();

        if !is_casing_candidate(
            trimmed,
        ) {
            continue;
        }

        candidate_count += 1;

        if let Some(style) =
            detect_case_style(
                trimmed,
            )
        {
            *counts
                .entry(style)
                .or_insert(0) += 1;
        }
    }

    if candidate_count
        < MIN_CASING_VALUES
    {
        return None;
    }

    let (
        dominant_style,
        dominant_count,
    ) =
        counts
            .into_iter()
            .max_by_key(
                |(_, count)| {
                    *count
                },
            )?;

    let confidence =
        dominant_count as f32
            / candidate_count as f32;

    let affected_rows =
        candidate_count
            .saturating_sub(
                dominant_count,
            );

    Some((
        dominant_style,
        confidence,
        affected_rows,
    ))
}

fn detect_case_style(
    value: &str,
) -> Option<CaseStyle> {
    let letters: Vec<char> =
        value
            .chars()
            .filter(|c| {
                c.is_alphabetic()
            })
            .collect();

    if letters.is_empty() {
        return None;
    }

    if letters
        .iter()
        .all(|c| {
            c.is_uppercase()
        })
    {
        return Some(
            CaseStyle::Upper,
        );
    }

    if letters
        .iter()
        .all(|c| {
            c.is_lowercase()
        })
    {
        return Some(
            CaseStyle::Lower,
        );
    }

    if is_title_case(value) {
        return Some(
            CaseStyle::Title,
        );
    }

    None
}

fn is_title_case(
    value: &str,
) -> bool {
    value
        .split_whitespace()
        .all(|word| {
            is_title_case_word(
                word,
            )
        })
}

fn is_title_case_word(
    word: &str,
) -> bool {
    let mut letters =
        word
            .chars()
            .filter(|c| {
                c.is_alphabetic()
            });

    let Some(first) =
        letters.next()
    else {
        return false;
    };

    if !first.is_uppercase() {
        return false;
    }

    letters.all(|c| {
        c.is_lowercase()
    })
}

fn apply_case_style(
    value: &str,
    style: CaseStyle,
) -> String {
    match style {
        CaseStyle::Lower => {
            value.to_lowercase()
        }

        CaseStyle::Upper => {
            value.to_uppercase()
        }

        CaseStyle::Title => {
            to_title_case(value)
        }
    }
}

fn to_title_case(
    value: &str,
) -> String {
    value
        .split_whitespace()
        .map(title_case_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case_word(
    word: &str,
) -> String {
    let mut chars =
        word.chars();

    let Some(first) =
        chars.next()
    else {
        return String::new();
    };

    let first =
        first
            .to_uppercase()
            .collect::<String>();

    let rest =
        chars
            .collect::<String>()
            .to_lowercase();

    format!(
        "{first}{rest}"
    )
}

fn is_casing_candidate(
    value: &str,
) -> bool {
    if value.is_empty() {
        return false;
    }

    // Jangan sentuh identifier,
    // nomor, currency, percentage.
    if value
        .chars()
        .any(|c| {
            c.is_ascii_digit()
                || matches!(
                    c,
                    '%' | '$' | '€' | '£'
                )
        })
    {
        return false;
    }

    value
        .chars()
        .any(|c| {
            c.is_alphabetic()
        })
}

fn has_case_variant_collisions(
    series: &Series,
) -> bool {
    let Ok(strings) =
        series.str()
    else {
        return false;
    };

    let mut variants:
        HashMap<
            String,
            HashSet<String>,
        > =
        HashMap::new();

    for index in 0..strings.len() {
        let Some(value) =
            strings.get(index)
        else {
            continue;
        };

        let trimmed =
            value.trim();

        if !is_casing_candidate(
            trimmed,
        ) {
            continue;
        }

        let key =
            trimmed
                .to_lowercase();

        variants
            .entry(key)
            .or_default()
            .insert(
                trimmed.to_string(),
            );
    }

    variants
        .values()
        .any(|forms| {
            forms.len() > 1
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_title_case_column_with_outlier() {
        let series =
            Series::new(
                "name".into(),
                [
                    "Alice",
                    "Bob",
                    "Charlie",
                    "David",
                    "edward",
                ],
            );

        let suggestion =
            analyze_casing(
                &series,
            )
            .expect(
                "Casing inconsistency should be detected",
            );

        assert_eq!(
            suggestion.kind,
            CleaningKind::NormalizeCasing
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
    fn applies_title_case_normalization() {
        let series =
            Series::new(
                "name".into(),
                [
                    "Alice",
                    "Bob",
                    "Charlie",
                    "David",
                    "edward",
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_casing_normalization(
                &series,
            )
            .expect(
                "Casing normalization should work",
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
            strings.get(4),
            Some("Edward")
        );
    }

    #[test]
    fn supports_uppercase_dominant_style() {
        let series =
            Series::new(
                "status".into(),
                [
                    "ACTIVE",
                    "PENDING",
                    "CLOSED",
                    "OPEN",
                    "cancelled",
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_casing_normalization(
                &series,
            )
            .expect(
                "Uppercase normalization should work",
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
            strings.get(4),
            Some("CANCELLED")
        );
    }

    #[test]
    fn ignores_case_variant_category_collisions() {
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

        assert!(
            analyze_casing(
                &series,
            )
            .is_none()
        );
    }

    #[test]
    fn ignores_small_sample() {
        let series =
            Series::new(
                "name".into(),
                [
                    "Alice",
                    "bob",
                ],
            );

        assert!(
            analyze_casing(
                &series,
            )
            .is_none()
        );
    }
}