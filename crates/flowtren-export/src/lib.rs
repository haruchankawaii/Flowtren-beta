use std::{
    error::Error,
    fmt,
    io,
};

use polars::prelude::PolarsError;
use rust_xlsxwriter::XlsxError;

pub mod cleaned_csv;
pub mod cleaned_xlsx;
pub mod metadata;

pub use cleaned_csv::{
    export_cleaned_csv,
};

pub use cleaned_xlsx::{
    export_cleaned_xlsx,
};

pub use metadata::{
    ExportFormat,
    ExportMetadata,
};

pub type ExportResult<T> =
    Result<T, ExportError>;

#[derive(Debug)]
pub enum ExportError {
    Io(
        io::Error,
    ),

    Polars(
        PolarsError,
    ),

    Xlsx(
        XlsxError,
    ),

    InvalidData(
        String,
    ),
}

impl fmt::Display
    for ExportError
{
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            ExportError::Io(
                error,
            ) => {
                write!(
                    formatter,
                    "I/O error: {error}"
                )
            }

            ExportError::Polars(
                error,
            ) => {
                write!(
                    formatter,
                    "Polars error: {error}"
                )
            }

            ExportError::Xlsx(
                error,
            ) => {
                write!(
                    formatter,
                    "XLSX error: {error}"
                )
            }

            ExportError::InvalidData(
                message,
            ) => {
                write!(
                    formatter,
                    "Invalid export data: {message}"
                )
            }
        }
    }
}

impl Error
    for ExportError
{
    fn source(
        &self,
    ) -> Option<
        &(dyn Error + 'static)
    > {
        match self {
            ExportError::Io(
                error,
            ) => {
                Some(error)
            }

            ExportError::Polars(
                error,
            ) => {
                Some(error)
            }

            ExportError::Xlsx(
                error,
            ) => {
                Some(error)
            }

            ExportError::InvalidData(
                _,
            ) => {
                None
            }
        }
    }
}

impl From<io::Error>
    for ExportError
{
    fn from(
        error: io::Error,
    ) -> Self {
        Self::Io(
            error,
        )
    }
}

impl From<PolarsError>
    for ExportError
{
    fn from(
        error: PolarsError,
    ) -> Self {
        Self::Polars(
            error,
        )
    }
}

impl From<XlsxError>
    for ExportError
{
    fn from(
        error: XlsxError,
    ) -> Self {
        Self::Xlsx(
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_data_error_has_message() {
        let error =
            ExportError::InvalidData(
                "test".to_string(),
            );

        assert_eq!(
            error.to_string(),
            "Invalid export data: test"
        );
    }
}