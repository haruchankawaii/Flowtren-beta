use std::path::Path;

use polars::prelude::*;

use crate::error::IoError;

pub fn read_csv(path: &Path) -> Result<DataFrame, IoError> {
    if !path.exists() {
        return Err(IoError::FileNotFound);
    }

    CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(path.to_path_buf()))
        .map_err(|err| IoError::Csv(err.to_string()))?
        .finish()
        .map_err(|err| IoError::Csv(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn reads_basic_csv() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/csv/basic.csv");

        let df = read_csv(&path).expect("CSV should load");

        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 3);
    }
}