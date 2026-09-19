use std::{
    error::Error,
    fmt,
};

use serde::{
    Serialize,
    Serializer,
};

#[derive(Debug)]
pub enum AppError {
    InvalidPath(
        String,
    ),

    UnsupportedFileType(
        String,
    ),

    DatasetNotLoaded,

    Io(
        String,
    ),

    Profile(
        String,
    ),

    Cleaning(
        String,
    ),

    InvalidCleaningSuggestion(
        String,
    ),

    Quality(
        String,
    ),

    Statistics(
        String,
    ),

    Insights(
        String,
    ),

    Charts(
        String,
    ),

    Export(
        String,
    ),

    License(
        String,
    ),

    State(
        String,
    ),

    Internal(
        String,
    ),
}

impl AppError {
    pub fn io(
        error: impl ToString,
    ) -> Self {
        Self::Io(
            error.to_string(),
        )
    }

    pub fn profile(
        error: impl ToString,
    ) -> Self {
        Self::Profile(
            error.to_string(),
        )
    }

    pub fn cleaning(
        error: impl ToString,
    ) -> Self {
        Self::Cleaning(
            error.to_string(),
        )
    }

    pub fn quality(
        error: impl ToString,
    ) -> Self {
        Self::Quality(
            error.to_string(),
        )
    }

    pub fn statistics(
        error: impl ToString,
    ) -> Self {
        Self::Statistics(
            error.to_string(),
        )
    }

    pub fn insights(
        error: impl ToString,
    ) -> Self {
        Self::Insights(
            error.to_string(),
        )
    }

    pub fn charts(
        error: impl ToString,
    ) -> Self {
        Self::Charts(
            error.to_string(),
        )
    }

    pub fn export(
        error: impl ToString,
    ) -> Self {
        Self::Export(
            error.to_string(),
        )
    }

    pub fn license(
        error: impl ToString,
    ) -> Self {
        Self::License(
            error.to_string(),
        )
    }

    pub fn state(
        error: impl ToString,
    ) -> Self {
        Self::State(
            error.to_string(),
        )
    }
}

impl fmt::Display
    for AppError
{
    fn fmt(
        &self,
        formatter:
            &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            AppError::InvalidPath(
                message,
            ) => {
                write!(
                    formatter,
                    "Invalid path: {message}"
                )
            }

            AppError::UnsupportedFileType(
                extension,
            ) => {
                write!(
                    formatter,
                    "Unsupported file type: {extension}"
                )
            }

            AppError::DatasetNotLoaded => {
                write!(
                    formatter,
                    "No dataset is currently loaded"
                )
            }

            AppError::Io(
                message,
            ) => {
                write!(
                    formatter,
                    "Failed to read dataset: {message}"
                )
            }

            AppError::Profile(
                message,
            ) => {
                write!(
                    formatter,
                    "Failed to profile dataset: {message}"
                )
            }

            AppError::Cleaning(
                message,
            ) => {
                write!(
                    formatter,
                    "Failed to clean dataset: {message}"
                )
            }

            AppError::InvalidCleaningSuggestion(
                message,
            ) => {
                write!(
                    formatter,
                    "Invalid cleaning suggestion: {message}"
                )
            }

            AppError::Quality(
                message,
            ) => {
                write!(
                    formatter,
                    "Failed to analyze data quality: {message}"
                )
            }

            AppError::Statistics(
                message,
            ) => {
                write!(
                    formatter,
                    "Failed to calculate statistics: {message}"
                )
            }

            AppError::Insights(
                message,
            ) => {
                write!(
                    formatter,
                    "Failed to generate insights: {message}"
                )
            }

            AppError::Charts(
                message,
            ) => {
                write!(
                    formatter,
                    "Failed to recommend charts: {message}"
                )
            }

            AppError::Export(
                message,
            ) => {
                write!(
                    formatter,
                    "Failed to export dataset: {message}"
                )
            }

            AppError::License(
                message,
            ) => {
                write!(
                    formatter,
                    "License error: {message}"
                )
            }

            AppError::State(
                message,
            ) => {
                write!(
                    formatter,
                    "Application state error: {message}"
                )
            }

            AppError::Internal(
                message,
            ) => {
                write!(
                    formatter,
                    "Internal error: {message}"
                )
            }
        }
    }
}

impl Error
    for AppError
{
}

impl Serialize
    for AppError
{
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<
        S::Ok,
        S::Error,
    >
    where
        S: Serializer,
    {
        serializer.serialize_str(
            &self.to_string(),
        )
    }
}