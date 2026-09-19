use std::path::{
    Path,
    PathBuf,
};

use serde::Serialize;
use tauri::State;

use flowtren_io::{
    csv::reader::read_csv,
    xlsx::reader::read_xlsx_first_sheet,
};

use crate::{
    error::AppError,
    state::{
        DatasetState,
        LoadedDataset,
    },
};

const BETA_MAX_ROWS: usize =
    50_000;

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(
    rename_all = "camelCase"
)]
pub struct OpenDatasetResponse {
    pub file_name: String,

    pub file_path: String,

    pub file_type: String,

    pub row_count: usize,

    pub column_count: usize,
}

#[tauri::command]
pub fn open_dataset(
    path: String,
    state: State<'_, DatasetState>,
) -> Result<
    OpenDatasetResponse,
    AppError,
> {
    let path =
        PathBuf::from(
            &path,
        );

    validate_path(
        &path,
    )?;

    let extension =
        file_extension(
            &path,
        )?;

    let dataframe =
        match extension.as_str() {
            "csv" => {
                read_csv(
                    &path,
                )
                .map_err(
                    AppError::io,
                )?
            }

            "xlsx" => {
                read_xlsx_first_sheet(
                    &path,
                )
                .map_err(
                    AppError::io,
                )?
            }

            _ => {
                return Err(
                    AppError::UnsupportedFileType(
                        extension,
                    ),
                );
            }
        };

    enforce_beta_row_limit(
        dataframe.height(),
    )?;

    let row_count =
        dataframe.height();

    let column_count =
        dataframe.width();

    let file_name =
        path.file_name()
            .and_then(
                |name| {
                    name.to_str()
                },
            )
            .ok_or_else(
                || {
                    AppError::InvalidPath(
                        "Could not determine file name"
                            .to_string(),
                    )
                },
            )?
            .to_string();

    let file_path =
        path.to_string_lossy()
            .to_string();

    state
        .replace(
            LoadedDataset::new(
                dataframe,

                path.clone(),

                file_name.clone(),
            ),
        )
        .map_err(
            AppError::state,
        )?;

    Ok(
        OpenDatasetResponse {
            file_name,

            file_path,

            file_type:
                extension,

            row_count,

            column_count,
        },
    )
}

fn validate_path(
    path: &Path,
) -> Result<(), AppError> {
    if !path.exists() {
        return Err(
            AppError::InvalidPath(
                "File does not exist"
                    .to_string(),
            ),
        );
    }

    if !path.is_file() {
        return Err(
            AppError::InvalidPath(
                "Path is not a file"
                    .to_string(),
            ),
        );
    }

    Ok(())
}

fn file_extension(
    path: &Path,
) -> Result<String, AppError> {
    path.extension()
        .and_then(
            |extension| {
                extension.to_str()
            },
        )
        .map(
            |extension| {
                extension
                    .to_lowercase()
            },
        )
        .ok_or_else(
            || {
                AppError::InvalidPath(
                    "File has no extension"
                        .to_string(),
                )
            },
        )
}

fn enforce_beta_row_limit(
    row_count: usize,
) -> Result<(), AppError> {
    if row_count
        > BETA_MAX_ROWS
    {
        return Err(
            AppError::InvalidPath(
                format!(
                    "Flowtren Beta supports datasets up to {BETA_MAX_ROWS} rows. This file contains {row_count} rows.",
                ),
            ),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_csv_extension() {
        let path =
            PathBuf::from(
                "sample.CSV",
            );

        assert_eq!(
            file_extension(
                &path,
            )
            .unwrap(),
            "csv"
        );
    }

    #[test]
    fn detects_xlsx_extension() {
        let path =
            PathBuf::from(
                "sample.XLSX",
            );

        assert_eq!(
            file_extension(
                &path,
            )
            .unwrap(),
            "xlsx"
        );
    }

    #[test]
    fn missing_extension_is_error() {
        let path =
            PathBuf::from(
                "sample",
            );

        assert!(
            file_extension(
                &path,
            )
            .is_err()
        );
    }

    #[test]
    fn beta_accepts_dataset_at_limit() {
        assert!(
            enforce_beta_row_limit(
                50_000,
            )
            .is_ok()
        );
    }

    #[test]
    fn beta_rejects_dataset_above_limit() {
        let result =
            enforce_beta_row_limit(
                50_001,
            );

        assert!(
            result.is_err()
        );
    }

    #[test]
    fn beta_rejects_large_dataset() {
        let result =
            enforce_beta_row_limit(
                250_000,
            );

        let error =
            result
                .unwrap_err()
                .to_string();

        assert!(
            error.contains(
                "50,000"
            )
            || error.contains(
                "50000"
            )
        );
    }
}