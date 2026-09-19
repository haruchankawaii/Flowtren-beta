use std::collections::HashMap;

use polars::prelude::*;

use crate::suggestion::{
    CleaningKind,
    CleaningSuggestion,
};

const MIN_CATEGORY_CONFIDENCE: f32 = 0.75;

pub fn analyze_categories(
    series: &Series,
) -> Option<CleaningSuggestion> {
    if series.dtype() != &DataType::String {
        return None;
    }

    let canonical_map =
        build_canonical_map(series)?;

    if canonical_map.is_empty() {
        return None;
    }

    let strings =
        series.str().ok()?;

    let mut affected_rows = 0usize;
    let mut grouped_rows = 0usize;
    let mut canonical_rows = 0usize;

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

        let key =
            normalize_key(trimmed);

        let Some(canonical) =
            canonical_map.get(&key)
        else {
            continue;
        };

        grouped_rows += 1;

        if trimmed == canonical {
            canonical_rows += 1;
        } else {
            affected_rows += 1;
        }
    }

    if affected_rows == 0
        || grouped_rows == 0
    {
        return None;
    }

    let confidence =
        canonical_rows as f32
            / grouped_rows as f32;

    if confidence
        < MIN_CATEGORY_CONFIDENCE
    {
        return None;
    }

    Some(
        CleaningSuggestion::new(
            series.name().to_string(),
            CleaningKind::NormalizeCategories,
            affected_rows,
            confidence,
        ),
    )
}

pub fn apply_category_normalization(
    series: &Series,
) -> PolarsResult<(Column, usize)> {
    if series.dtype() != &DataType::String {
        return Ok((
            series.clone().into_column(),
            0,
        ));
    }

    let Some(canonical_map) =
        build_canonical_map(series)
    else {
        return Ok((
            series.clone().into_column(),
            0,
        ));
    };

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

                if trimmed.is_empty() {
                    return Some(
                        value.to_string(),
                    );
                }

                let key =
                    normalize_key(
                        trimmed,
                    );

                let Some(canonical) =
                    canonical_map
                        .get(&key)
                else {
                    return Some(
                        value.to_string(),
                    );
                };

                if trimmed
                    != canonical
                {
                    affected_rows += 1;

                    return Some(
                        canonical.clone(),
                    );
                }

                Some(
                    value.to_string(),
                )
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

fn build_canonical_map(
    series: &Series,
) -> Option<HashMap<String, String>> {
    if series.dtype()
        != &DataType::String
    {
        return None;
    }

    let strings =
        series.str().ok()?;

    // normalized key
    //   ↓
    // exact spelling -> count
    //
    // jakarta:
    //   Jakarta -> 3
    //   jakarta -> 1
    //   JAKARTA -> 1
    let mut groups:
        HashMap<
            String,
            HashMap<String, usize>,
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

        if trimmed.is_empty() {
            continue;
        }

        if !is_category_candidate(
            trimmed,
        ) {
            continue;
        }

        let key =
            normalize_key(trimmed);

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

    let mut canonical_map =
        HashMap::new();

    for (
        key,
        variants,
    ) in groups
    {
        // Tidak ada inconsistency.
        if variants.len() < 2 {
            continue;
        }

        let mut ranked:
            Vec<(String, usize)> =
            variants
                .into_iter()
                .collect();

        ranked.sort_by(
            |a, b| {
                b.1.cmp(&a.1)
                    .then_with(
                        || {
                            a.0.cmp(
                                &b.0,
                            )
                        },
                    )
            },
        );

        let Some((
            winner,
            winner_count,
        )) =
            ranked.first()
        else {
            continue;
        };

        // Jika dua bentuk paling populer
        // memiliki jumlah sama, kita tidak
        // boleh menebak canonical value.
        if ranked.len() > 1
            && ranked[1].1
                == *winner_count
        {
            continue;
        }

        canonical_map.insert(
            key,
            winner.clone(),
        );
    }

    Some(canonical_map)
}

fn normalize_key(
    value: &str,
) -> String {
    value
        .trim()
        .to_lowercase()
}

fn is_category_candidate(
    value: &str,
) -> bool {
    if value.is_empty() {
        return false;
    }

    // Jangan sentuh nilai yang sangat
    // mungkin berupa number, currency,
    // percentage atau identifier.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_category_case_inconsistency() {
        let series =
            Series::new(
                "city".into(),
                [
                    "Jakarta",
                    "Jakarta",
                    "Jakarta",
                    "jakarta",
                    "Bandung",
                ],
            );

        let suggestion =
            analyze_categories(
                &series,
            )
            .expect(
                "Category inconsistency should be detected",
            );

        assert_eq!(
            suggestion.kind,
            CleaningKind::NormalizeCategories
        );

        assert_eq!(
            suggestion.affected_rows,
            1
        );

        assert!(
            suggestion.confidence
                >= 0.75
        );
    }

    #[test]
    fn applies_category_normalization() {
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

        let (
            column,
            affected_rows,
        ) =
            apply_category_normalization(
                &series,
            )
            .expect(
                "Category normalization should work",
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
            Some("Jakarta")
        );

        assert_eq!(
            strings.get(3),
            Some("Jakarta")
        );
    }

    #[test]
    fn ignores_ambiguous_category_tie() {
        let series =
            Series::new(
                "city".into(),
                [
                    "Jakarta",
                    "jakarta",
                ],
            );

        assert!(
            analyze_categories(
                &series,
            )
            .is_none()
        );
    }

    #[test]
    fn ignores_numeric_like_values() {
        let series =
            Series::new(
                "code".into(),
                [
                    "ABC1",
                    "abc1",
                    "ABC1",
                ],
            );

        assert!(
            analyze_categories(
                &series,
            )
            .is_none()
        );
    }
}