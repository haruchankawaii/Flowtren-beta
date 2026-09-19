use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("Failed to read CSV: {0}")]
    Csv(String),

    #[error("Failed to read XLSX: {0}")]
    Xlsx(String),

    #[error("File not found")]
    FileNotFound,
}

