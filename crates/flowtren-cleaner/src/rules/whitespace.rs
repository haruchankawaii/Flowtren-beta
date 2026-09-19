use polars::prelude::*;

use crate::suggestion::{
    CleaningKind,
    CleaningSuggestion,
};

pub fn analyze_whitespace(
    series: &Series,
) -> Option<CleaningSuggestion> {
    if series.dtype() != &DataType::String {
        return None;
    }

    let Ok(strings) = series.str() else {
        return None;
    };

    let mut affected_rows = 0usize;

    for index in 0..strings.len() {
        let Some(value) = strings.get(index) else {
            continue;
        };

        if value != value.trim() {
            affected_rows += 1;
        }
    }

    if affected_rows == 0 {
        return None;
    }

    Some(
        CleaningSuggestion::new(
            series.name().to_string(),
            CleaningKind::TrimWhitespace,
            affected_rows,
            1.0,
        ),
    )
}

pub fn apply_trim_whitespace(
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

                let trimmed = value.trim();

                if value != trimmed {
                    affected_rows += 1;
                }

                Some(trimmed.to_string())
            })
            .collect();

    let column = Column::new(
        series.name().clone(),
        values,
    );

    Ok((column, affected_rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_whitespace() {
        let series = Series::new(
            "region".into(),
            [
                " Jakarta ",
                "Bandung",
                "Surabaya ",
            ],
        );

        let suggestion =
            analyze_whitespace(&series)
                .expect(
                    "Whitespace should be detected",
                );

        assert_eq!(
            suggestion.kind,
            CleaningKind::TrimWhitespace
        );

        assert_eq!(
            suggestion.affected_rows,
            2
        );
    }

    #[test]
    fn ignores_clean_strings() {
        let series = Series::new(
            "region".into(),
            [
                "Jakarta",
                "Bandung",
            ],
        );

        assert!(
            analyze_whitespace(&series)
                .is_none()
        );
    }

    #[test]
    fn applies_whitespace_cleaning() {
        let series = Series::new(
            "region".into(),
            [
                Some(" Jakarta "),
                Some("Bandung"),
                None,
                Some("Surabaya "),
            ],
        );

        let (column, affected_rows) =
            apply_trim_whitespace(&series)
                .expect(
                    "Whitespace cleaning should work",
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
            Some("Bandung")
        );

        assert_eq!(
            strings.get(2),
            None
        );

        assert_eq!(
            strings.get(3),
            Some("Surabaya")
        );
    }
}