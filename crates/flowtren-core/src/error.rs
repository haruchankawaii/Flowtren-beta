#[derive(Debug)]
pub enum FlowtrenError {
    InvalidFile,
    UnsupportedFormat,
    DataProcessing(String),
}