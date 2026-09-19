use serde::Serialize;
use tauri::State;

use flowtren_cleaner::{
    audit::{
        CleaningAudit,
        CleaningAuditEntry,
    },
    engine::{
        analyze_dataframe,
        apply_suggestions,
    },
    suggestion::{
        CleaningKind,
        CleaningSuggestion,
    },
};

use crate::{
    error::AppError,
    state::DatasetState,
};

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct CleaningSuggestionResponse {
    pub id:
        usize,

    pub column:
        String,

    pub kind:
        String,

    pub affected_rows:
        usize,

    pub confidence:
        f32,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct CleaningSuggestionsResponse {
    pub suggestion_count:
        usize,

    pub total_affected_rows:
        usize,

    pub suggestions:
        Vec<
            CleaningSuggestionResponse,
        >,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct CleaningAuditEntryResponse {
    pub column:
        String,

    pub kind:
        String,

    pub affected_rows:
        usize,
}

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct ApplyCleaningResponse {
    pub applied_suggestions:
        usize,

    pub audit_entries:
        Vec<
            CleaningAuditEntryResponse,
        >,

    pub total_changes:
        usize,

    pub row_count_before:
        usize,

    pub row_count_after:
        usize,

    pub column_count:
        usize,
}

#[tauri::command]
pub fn suggest_cleaning(
    state:
        State<'_, DatasetState>,
) -> Result<
    CleaningSuggestionsResponse,
    AppError,
> {
    let (
        dataframe,
        dataset_path,
    ) =
        state
            .cleaning_snapshot()
            .map_err(
                map_cleaning_state_error,
            )?;

    let suggestions =
        analyze_dataframe(
            &dataframe,
        )
        .map_err(
            |error| {
                AppError::Cleaning(
                    error
                        .to_string(),
                )
            },
        )?;

    let response =
        suggestions_response(
            &suggestions,
        );

    state
        .store_cleaning_suggestions(
            &dataset_path,
            suggestions,
        )
        .map_err(
            map_cleaning_state_error,
        )?;

    Ok(
        response,
    )
}

#[tauri::command]
pub fn apply_cleaning(
    suggestion_ids:
        Option<Vec<usize>>,

    state:
        State<'_, DatasetState>,
) -> Result<
    ApplyCleaningResponse,
    AppError,
> {
    state
        .with_dataset_mut(
            |dataset| {
                if dataset
                    .cleaning_suggestions
                    .is_empty()
                {
                    return Err(
                        "No cleaning suggestions are available. Run suggest_cleaning first."
                            .to_string(),
                    );
                }

                let selected =
                    select_suggestions(
                        &dataset
                            .cleaning_suggestions,

                        suggestion_ids
                            .as_deref(),
                    )?;

                let row_count_before =
                    dataset
                        .dataframe
                        .height();

                let applied_suggestions =
                    selected
                        .len();

                let result =
                    apply_suggestions(
                        &dataset
                            .dataframe,

                        &selected,
                    )
                    .map_err(
                        |error| {
                            error
                                .to_string()
                        },
                    )?;

                let row_count_after =
                    result
                        .dataframe
                        .height();

                let column_count =
                    result
                        .dataframe
                        .width();

                let response =
                    cleaning_result_response(
                        applied_suggestions,

                        row_count_before,

                        row_count_after,

                        column_count,

                        &result
                            .audit,
                    );

                dataset
                    .dataframe =
                    result
                        .dataframe;

                dataset
                    .cleaning_suggestions
                    .clear();

                Ok(
                    response,
                )
            },
        )
        .map_err(
            map_cleaning_state_error,
        )
}

fn suggestions_response(
    suggestions:
        &[CleaningSuggestion],
) -> CleaningSuggestionsResponse {
    let total_affected_rows =
        suggestions
            .iter()
            .map(
                |suggestion| {
                    suggestion
                        .affected_rows
                },
            )
            .sum();

    let suggestions:
        Vec<
            CleaningSuggestionResponse,
        > =
        suggestions
            .iter()
            .enumerate()
            .map(
                |(
                    index,
                    suggestion,
                )| {
                    CleaningSuggestionResponse {
                        id:
                            index,

                        column:
                            suggestion
                                .column
                                .clone(),

                        kind:
                            cleaning_kind_name(
                                &suggestion
                                    .kind,
                            )
                            .to_string(),

                        affected_rows:
                            suggestion
                                .affected_rows,

                        confidence:
                            suggestion
                                .confidence,
                    }
                },
            )
            .collect();

    CleaningSuggestionsResponse {
        suggestion_count:
            suggestions
                .len(),

        total_affected_rows,

        suggestions,
    }
}

fn select_suggestions(
    suggestions:
        &[CleaningSuggestion],

    ids:
        Option<&[usize]>,
) -> Result<
    Vec<
        CleaningSuggestion,
    >,
    String,
> {
    let Some(
        ids,
    ) =
        ids
    else {
        return Ok(
            suggestions
                .to_vec(),
        );
    };

    let mut selected =
        Vec::with_capacity(
            ids.len(),
        );

    for id in ids {
        let suggestion =
            suggestions
                .get(
                    *id,
                )
                .ok_or_else(
                    || {
                        format!(
                            "Cleaning suggestion id {id} does not exist"
                        )
                    },
                )?;

        selected.push(
            suggestion
                .clone(),
        );
    }

    Ok(
        selected,
    )
}

fn cleaning_result_response(
    applied_suggestions:
        usize,

    row_count_before:
        usize,

    row_count_after:
        usize,

    column_count:
        usize,

    audit:
        &CleaningAudit,
) -> ApplyCleaningResponse {
    ApplyCleaningResponse {
        applied_suggestions,

        audit_entries:
            audit
                .entries
                .iter()
                .map(
                    audit_entry_response,
                )
                .collect(),

        total_changes:
            audit
                .total_changes(),

        row_count_before,

        row_count_after,

        column_count,
    }
}

fn audit_entry_response(
    entry:
        &CleaningAuditEntry,
) -> CleaningAuditEntryResponse {
    CleaningAuditEntryResponse {
        column:
            entry
                .column
                .clone(),

        kind:
            cleaning_kind_name(
                &entry
                    .kind,
            )
            .to_string(),

        affected_rows:
            entry
                .affected_rows,
    }
}

fn cleaning_kind_name(
    kind:
        &CleaningKind,
) -> &'static str {
    match kind {
        CleaningKind::TrimWhitespace => {
            "trimWhitespace"
        }

        CleaningKind::EmptyStringToNull => {
            "emptyStringToNull"
        }

        CleaningKind::NormalizeMissingMarker => {
            "normalizeMissingMarker"
        }

        CleaningKind::RemoveDuplicateRows => {
            "removeDuplicateRows"
        }

        CleaningKind::NormalizeNumberFormat => {
            "normalizeNumberFormat"
        }

        CleaningKind::NormalizeCurrency => {
            "normalizeCurrency"
        }

        CleaningKind::NormalizePercentage => {
            "normalizePercentage"
        }

        CleaningKind::NormalizeDate => {
            "normalizeDate"
        }

        CleaningKind::RemoveEmptyRows => {
            "removeEmptyRows"
        }

        CleaningKind::NormalizeCategories => {
            "normalizeCategories"
        }

        CleaningKind::NormalizeCasing => {
            "normalizeCasing"
        }
    }
}

fn map_cleaning_state_error(
    error:
        String,
) -> AppError {
    if error
        == "No dataset is currently loaded"
    {
        AppError::DatasetNotLoaded
    } else if error
        .starts_with(
            "Cleaning suggestion id",
        )
    {
        AppError::InvalidCleaningSuggestion(
            error,
        )
    } else {
        AppError::Cleaning(
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleaning_kind_names_are_stable() {
        assert_eq!(
            cleaning_kind_name(
                &CleaningKind::TrimWhitespace,
            ),
            "trimWhitespace"
        );

        assert_eq!(
            cleaning_kind_name(
                &CleaningKind::RemoveDuplicateRows,
            ),
            "removeDuplicateRows"
        );

        assert_eq!(
            cleaning_kind_name(
                &CleaningKind::NormalizeCasing,
            ),
            "normalizeCasing"
        );
    }

    #[test]
    fn selects_all_when_ids_are_none() {
        let suggestions =
            vec![
                CleaningSuggestion::new(
                    "name",
                    CleaningKind::TrimWhitespace,
                    2,
                    1.0,
                ),

                CleaningSuggestion::new(
                    "city",
                    CleaningKind::NormalizeCasing,
                    1,
                    0.9,
                ),
            ];

        let selected =
            select_suggestions(
                &suggestions,
                None,
            )
            .unwrap();

        assert_eq!(
            selected.len(),
            2
        );
    }

    #[test]
    fn selects_requested_suggestions() {
        let suggestions =
            vec![
                CleaningSuggestion::new(
                    "name",
                    CleaningKind::TrimWhitespace,
                    2,
                    1.0,
                ),

                CleaningSuggestion::new(
                    "city",
                    CleaningKind::NormalizeCasing,
                    1,
                    0.9,
                ),
            ];

        let selected =
            select_suggestions(
                &suggestions,

                Some(
                    &[1],
                ),
            )
            .unwrap();

        assert_eq!(
            selected.len(),
            1
        );

        assert_eq!(
            selected[0]
                .column,
            "city"
        );
    }

    #[test]
    fn invalid_suggestion_id_is_rejected() {
        let suggestions =
            vec![
                CleaningSuggestion::new(
                    "name",
                    CleaningKind::TrimWhitespace,
                    2,
                    1.0,
                ),
            ];

        assert!(
            select_suggestions(
                &suggestions,

                Some(
                    &[99],
                ),
            )
            .is_err()
        );
    }
}