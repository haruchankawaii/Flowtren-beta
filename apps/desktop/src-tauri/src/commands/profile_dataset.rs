use serde::Serialize;
use tauri::State;

use flowtren_core::semantic_type::SemanticType;

use flowtren_profiler::dataset_profile::{
    profile_dataframe,
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
pub struct DatasetProfileResponse {
    pub file_name:
        String,

    pub row_count:
        usize,

    pub column_count:
        usize,

    pub columns:
        Vec<
            ColumnProfileResponse,
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
pub struct ColumnProfileResponse {
    pub name:
        String,

    pub dtype:
        String,

    pub semantic_type:
        String,

    pub semantic_confidence:
        f32,

    pub null_count:
        usize,

    pub unique_count:
        usize,
}

#[tauri::command]
pub fn profile_dataset(
    state:
        State<'_, DatasetState>,
) -> Result<
    DatasetProfileResponse,
    AppError,
> {
    let (
        dataframe,
        file_name,
    ) =
        state
            .profile_snapshot()
            .map_err(
                map_profile_state_error,
            )?;

    let profile =
        profile_dataframe(
            &dataframe,
        )
        .map_err(
            |error| {
                AppError::profile(
                    error
                        .to_string(),
                )
            },
        )?;

    let columns =
        profile
            .columns
            .into_iter()
            .map(
                |column| {
                    ColumnProfileResponse {
                        name:
                            column.name,

                        dtype:
                            column.dtype,

                        semantic_type:
                            semantic_type_name(
                                &column
                                    .semantic_type,
                            )
                            .to_string(),

                        semantic_confidence:
                            column
                                .semantic_confidence,

                        null_count:
                            column
                                .null_count,

                        unique_count:
                            column
                                .unique_count,
                    }
                },
            )
            .collect();

    Ok(
        DatasetProfileResponse {
            file_name,

            row_count:
                profile
                    .row_count,

            column_count:
                profile
                    .column_count,

            columns,
        },
    )
}

fn semantic_type_name(
    semantic_type:
        &SemanticType,
) -> &'static str {
    match semantic_type {
        SemanticType::Integer => {
            "integer"
        }

        SemanticType::Decimal => {
            "decimal"
        }

        SemanticType::Currency => {
            "currency"
        }

        SemanticType::Percentage => {
            "percentage"
        }

        SemanticType::Category => {
            "category"
        }

        SemanticType::Boolean => {
            "boolean"
        }

        SemanticType::Date => {
            "date"
        }

        SemanticType::DateTime => {
            "datetime"
        }

        SemanticType::Email => {
            "email"
        }

        SemanticType::Phone => {
            "phone"
        }

        SemanticType::Identifier => {
            "identifier"
        }

        SemanticType::Text => {
            "text"
        }

        SemanticType::Unknown => {
            "unknown"
        }
    }
}

fn map_profile_state_error(
    error:
        String,
) -> AppError {
    if error
        == "No dataset is currently loaded"
    {
        AppError::DatasetNotLoaded
    } else {
        AppError::profile(
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_names_are_stable() {
        assert_eq!(
            semantic_type_name(
                &SemanticType::Integer,
            ),
            "integer"
        );

        assert_eq!(
            semantic_type_name(
                &SemanticType::Currency,
            ),
            "currency"
        );

        assert_eq!(
            semantic_type_name(
                &SemanticType::DateTime,
            ),
            "datetime"
        );

        assert_eq!(
            semantic_type_name(
                &SemanticType::Identifier,
            ),
            "identifier"
        );
    }
}