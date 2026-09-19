use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use serde::Serialize;
use tauri::State;

use flowtren_export::{
    export_cleaned_csv as write_cleaned_csv,
    export_cleaned_xlsx as write_cleaned_xlsx,
    ExportFormat,
    ExportMetadata,
};

use flowtren_quality::analyze_dataset_quality;

use crate::{
    error::AppError,
    state::DatasetState,
};

#[derive(
    Debug,
    Clone,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct ExportResponse {
    pub output_path: String,

    pub format: String,

    pub row_count: usize,

    pub column_count: usize,

    pub source_file: Option<String>,

    pub quality_score: Option<f64>,

    pub exported_at_unix_seconds: u64,
}

#[tauri::command]
pub fn export_dataset_csv(
    output_path: String,
    overwrite: Option<bool>,
    state: State<'_, DatasetState>,
) -> Result<
    ExportResponse,
    AppError,
> {
    export_dataset(
        output_path,
        overwrite.unwrap_or(false),
        ExportFormat::Csv,
        state,
    )
}

#[tauri::command]
pub fn export_dataset_xlsx(
    output_path: String,
    overwrite: Option<bool>,
    state: State<'_, DatasetState>,
) -> Result<
    ExportResponse,
    AppError,
> {
    export_dataset(
        output_path,
        overwrite.unwrap_or(false),
        ExportFormat::Xlsx,
        state,
    )
}

fn export_dataset(
    output_path: String,
    overwrite: bool,
    format: ExportFormat,
    state: State<'_, DatasetState>,
) -> Result<
    ExportResponse,
    AppError,
> {
    state
        .with_dataset(
            |dataset| {
                let output =
                    PathBuf::from(
                        &output_path,
                    );

                validate_output_path(
                    &dataset.path,
                    &output,
                    format,
                    overwrite,
                )
                .map_err(
                    |error| {
                        error.to_string()
                    },
                )?;

                match format {
                    ExportFormat::Csv => {
                        write_cleaned_csv(
                            &dataset.dataframe,
                            &output,
                        )
                        .map_err(
                            |error| {
                                error.to_string()
                            },
                        )?;
                    }

                    ExportFormat::Xlsx => {
                        write_cleaned_xlsx(
                            &dataset.dataframe,
                            &output,
                        )
                        .map_err(
                            |error| {
                                error.to_string()
                            },
                        )?;
                    }
                }

                let quality_score =
                    analyze_dataset_quality(
                        &dataset.dataframe,
                    )
                    .ok()
                    .map(
                        |quality| {
                            quality.score
                        },
                    );

                let mut metadata =
                    ExportMetadata::from_dataframe(
                        &dataset.dataframe,

                        output
                            .to_string_lossy()
                            .to_string(),

                        format,
                    )
                    .with_source_file(
                        dataset
                            .file_name
                            .clone(),
                    );

                if let Some(
                    quality_score,
                ) = quality_score
                {
                    metadata =
                        metadata
                            .with_quality_score(
                                quality_score,
                            );
                }

                Ok(
                    export_response(
                        &metadata,
                    ),
                )
            },
        )
        .map_err(
            map_export_error,
        )
}

fn validate_output_path(
    source_path: &Path,
    output_path: &Path,
    format: ExportFormat,
    overwrite: bool,
) -> Result<(), String> {
    if output_path
        .as_os_str()
        .is_empty()
    {
        return Err(
            "Output path cannot be empty"
                .to_string(),
        );
    }

    validate_extension(
        output_path,
        format,
    )?;

    let parent =
        output_path
            .parent()
            .ok_or_else(
                || {
                    "Output path has no parent directory"
                        .to_string()
                },
            )?;

    if !parent.exists() {
        return Err(
            format!(
                "Output directory does not exist: {}",
                parent.display(),
            ),
        );
    }

    if !parent.is_dir() {
        return Err(
            format!(
                "Output parent is not a directory: {}",
                parent.display(),
            ),
        );
    }

    if paths_refer_to_same_file(
        source_path,
        output_path,
    )? {
        return Err(
            "Flowtren will not overwrite the original dataset"
                .to_string(),
        );
    }

    if output_path.exists()
        && !overwrite
    {
        return Err(
            format!(
                "Output file already exists: {}",
                output_path.display(),
            ),
        );
    }

    Ok(())
}

fn validate_extension(
    path: &Path,
    format: ExportFormat,
) -> Result<(), String> {
    let extension =
        path.extension()
            .and_then(
                |extension| {
                    extension.to_str()
                },
            )
            .map(
                |extension| {
                    extension
                        .to_ascii_lowercase()
                },
            );

    let expected =
        format.as_str();

    match extension {
        Some(
            extension,
        ) if extension == expected => {
            Ok(())
        }

        Some(
            extension,
        ) => {
            Err(
                format!(
                    "Expected .{expected} output file, but received .{extension}",
                ),
            )
        }

        None => {
            Err(
                format!(
                    "Output file must have a .{expected} extension",
                ),
            )
        }
    }
}

fn paths_refer_to_same_file(
    left: &Path,
    right: &Path,
) -> Result<bool, String> {
    let left =
        absolute_normalized_path(
            left,
        )?;

    let right =
        absolute_normalized_path(
            right,
        )?;

    #[cfg(windows)]
    {
        return Ok(
            left.to_string_lossy()
                .eq_ignore_ascii_case(
                    &right
                        .to_string_lossy(),
                ),
        );
    }

    #[cfg(not(windows))]
    {
        Ok(
            left == right,
        )
    }
}

fn absolute_normalized_path(
    path: &Path,
) -> Result<PathBuf, String> {
    if path.exists() {
        return fs::canonicalize(
            path,
        )
        .map_err(
            |error| {
                format!(
                    "Could not resolve path '{}': {error}",
                    path.display(),
                )
            },
        );
    }

    let parent =
        path.parent()
            .ok_or_else(
                || {
                    format!(
                        "Path '{}' has no parent directory",
                        path.display(),
                    )
                },
            )?;

    let file_name =
        path.file_name()
            .ok_or_else(
                || {
                    format!(
                        "Path '{}' has no file name",
                        path.display(),
                    )
                },
            )?;

    let parent =
        fs::canonicalize(
            parent,
        )
        .map_err(
            |error| {
                format!(
                    "Could not resolve output directory '{}': {error}",
                    parent.display(),
                )
            },
        )?;

    Ok(
        parent.join(
            file_name,
        ),
    )
}

fn export_response(
    metadata: &ExportMetadata,
) -> ExportResponse {
    ExportResponse {
        output_path:
            metadata
                .output_file
                .clone(),

        format:
            metadata
                .format
                .as_str()
                .to_string(),

        row_count:
            metadata
                .row_count,

        column_count:
            metadata
                .column_count,

        source_file:
            metadata
                .source_file
                .clone(),

        quality_score:
            metadata
                .quality_score,

        exported_at_unix_seconds:
            metadata
                .exported_at_unix_seconds,
    }
}

fn map_export_error(
    error: String,
) -> AppError {
    if error
        == "No dataset is currently loaded"
    {
        AppError::DatasetNotLoaded
    } else {
        AppError::Export(
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{
        SystemTime,
        UNIX_EPOCH,
    };

    fn temporary_output(
        extension: &str,
    ) -> PathBuf {
        let unique =
            SystemTime::now()
                .duration_since(
                    UNIX_EPOCH,
                )
                .unwrap()
                .as_nanos();

        std::env::temp_dir()
            .join(
                format!(
                    "flowtren-export-test-{unique}.{extension}"
                ),
            )
    }

    #[test]
    fn accepts_csv_extension() {
        let path =
            PathBuf::from(
                "cleaned.csv",
            );

        assert!(
            validate_extension(
                &path,
                ExportFormat::Csv,
            )
            .is_ok()
        );
    }

    #[test]
    fn accepts_uppercase_csv_extension() {
        let path =
            PathBuf::from(
                "cleaned.CSV",
            );

        assert!(
            validate_extension(
                &path,
                ExportFormat::Csv,
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_wrong_csv_extension() {
        let path =
            PathBuf::from(
                "cleaned.xlsx",
            );

        assert!(
            validate_extension(
                &path,
                ExportFormat::Csv,
            )
            .is_err()
        );
    }

    #[test]
    fn accepts_xlsx_extension() {
        let path =
            PathBuf::from(
                "cleaned.xlsx",
            );

        assert!(
            validate_extension(
                &path,
                ExportFormat::Xlsx,
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_missing_extension() {
        let path =
            PathBuf::from(
                "cleaned",
            );

        assert!(
            validate_extension(
                &path,
                ExportFormat::Csv,
            )
            .is_err()
        );
    }

    #[test]
    fn detects_same_existing_file() {
        let path =
            temporary_output(
                "csv",
            );

        fs::write(
            &path,
            "test",
        )
        .unwrap();

        assert!(
            paths_refer_to_same_file(
                &path,
                &path,
            )
            .unwrap()
        );

        let _ =
            fs::remove_file(
                path,
            );
    }

    #[test]
    fn rejects_existing_file_without_overwrite() {
        let source =
            temporary_output(
                "csv",
            );

        let output =
            temporary_output(
                "csv",
            );

        fs::write(
            &source,
            "source",
        )
        .unwrap();

        fs::write(
            &output,
            "output",
        )
        .unwrap();

        let result =
            validate_output_path(
                &source,
                &output,
                ExportFormat::Csv,
                false,
            );

        assert!(
            result.is_err()
        );

        let _ =
            fs::remove_file(
                source,
            );

        let _ =
            fs::remove_file(
                output,
            );
    }

    #[test]
    fn permits_existing_file_when_overwrite_is_true() {
        let source =
            temporary_output(
                "csv",
            );

        let output =
            temporary_output(
                "csv",
            );

        fs::write(
            &source,
            "source",
        )
        .unwrap();

        fs::write(
            &output,
            "output",
        )
        .unwrap();

        let result =
            validate_output_path(
                &source,
                &output,
                ExportFormat::Csv,
                true,
            );

        assert!(
            result.is_ok()
        );

        let _ =
            fs::remove_file(
                source,
            );

        let _ =
            fs::remove_file(
                output,
            );
    }

    #[test]
    fn original_file_is_never_overwritten() {
        let source =
            temporary_output(
                "csv",
            );

        fs::write(
            &source,
            "source",
        )
        .unwrap();

        let result =
            validate_output_path(
                &source,
                &source,
                ExportFormat::Csv,
                true,
            );

        assert!(
            result.is_err()
        );

        let _ =
            fs::remove_file(
                source,
            );
    }
}