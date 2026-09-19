use chrono::NaiveDate;
use polars::prelude::*;

use crate::suggestion::{
    CleaningKind,
    CleaningSuggestion,
};

const MIN_DATE_CONFIDENCE: f32 = 0.80;

const DATE_FORMATS: &[&str] = &[
    "%Y-%m-%d",
    "%Y/%m/%d",
    "%d-%m-%Y",
    "%d/%m/%Y",
    "%m-%d-%Y",
    "%m/%d/%Y",
];

pub fn analyze_dates(
    series: &Series,
) -> Option<CleaningSuggestion> {
    if series.dtype() != &DataType::String {
        return None;
    }

    let Ok(strings) = series.str() else {
        return None;
    };

    let mut candidate_count = 0usize;
    let mut date_count = 0usize;
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

        let Some(date) =
            parse_date(trimmed)
        else {
            continue;
        };

        date_count += 1;

        let normalized =
            date.format("%Y-%m-%d").to_string();

        if normalized != trimmed {
            normalization_count += 1;
        }
    }

    if candidate_count == 0
        || date_count == 0
        || normalization_count == 0
    {
        return None;
    }

    let confidence =
        date_count as f32
            / candidate_count as f32;

    if confidence < MIN_DATE_CONFIDENCE {
        return None;
    }

    Some(
        CleaningSuggestion::new(
            series.name().to_string(),
            CleaningKind::NormalizeDate,
            normalization_count,
            confidence,
        ),
    )
}

pub fn apply_date_normalization(
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

                let Some(date) =
                    parse_date(trimmed)
                else {
                    return Some(
                        value.to_string(),
                    );
                };

                let normalized =
                    date
                        .format("%Y-%m-%d")
                        .to_string();

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

fn parse_date(
    value: &str,
) -> Option<NaiveDate> {
    DATE_FORMATS
        .iter()
        .find_map(|format| {
            NaiveDate::parse_from_str(
                value,
                format,
            )
            .ok()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_date_column() {
        let series =
            Series::new(
                "date".into(),
                [
                    "17/09/2026",
                    "18/09/2026",
                    "19/09/2026",
                ],
            );

        let suggestion =
            analyze_dates(&series)
                .expect(
                    "Date should be detected",
                );

        assert_eq!(
            suggestion.kind,
            CleaningKind::NormalizeDate
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
    fn applies_date_normalization() {
        let series =
            Series::new(
                "date".into(),
                [
                    Some("17/09/2026"),
                    Some("2026-09-18"),
                    Some("19-09-2026"),
                    None,
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_date_normalization(
                &series,
            )
            .expect(
                "Date normalization should work",
            );

        assert_eq!(
            affected_rows,
            2
        );

        let strings =
            column
                .as_materialized_series()
                .str()
                .unwrap();

        assert_eq!(
            strings.get(0),
            Some("2026-09-17")
        );

        assert_eq!(
            strings.get(1),
            Some("2026-09-18")
        );

        assert_eq!(
            strings.get(2),
            Some("2026-09-19")
        );

        assert_eq!(
            strings.get(3),
            None
        );
    }

    #[test]
    fn preserves_invalid_date() {
        let series =
            Series::new(
                "date".into(),
                [
                    "hello",
                    "17/09/2026",
                ],
            );

        let (
            column,
            affected_rows,
        ) =
            apply_date_normalization(
                &series,
            )
            .expect(
                "Date normalization should work",
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
            Some("hello")
        );

        assert_eq!(
            strings.get(1),
            Some("2026-09-17")
        );
    }
}