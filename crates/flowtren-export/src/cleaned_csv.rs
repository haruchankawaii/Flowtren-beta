use std::{
    fs::File,
    path::Path,
};

use polars::prelude::*;

use crate::{
    ExportResult,
};

pub fn export_cleaned_csv(
    dataframe: &DataFrame,
    path: impl AsRef<Path>,
) -> ExportResult<()> {
    let file =
        File::create(
            path,
        )?;

    let mut dataframe =
        dataframe.clone();

    CsvWriter::new(
        file,
    )
    .include_header(
        true,
    )
    .with_separator(
        b',',
    )
    .finish(
        &mut dataframe,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        fs,
        time::{
            SystemTime,
            UNIX_EPOCH,
        },
    };

    fn temporary_path()
        -> std::path::PathBuf
    {
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
                    "flowtren-export-{unique}.csv"
                ),
            )
    }

    #[test]
    fn exports_dataframe_to_csv() {
        let dataframe =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "name".into(),
                        [
                            "Alice",
                            "Bob",
                        ],
                    ),

                    Column::new(
                        "age".into(),
                        [
                            20_i64,
                            30_i64,
                        ],
                    ),
                ],
            )
            .unwrap();

        let path =
            temporary_path();

        export_cleaned_csv(
            &dataframe,
            &path,
        )
        .expect(
            "CSV export should succeed",
        );

        assert!(
            path.exists()
        );

        let content =
            fs::read_to_string(
                &path,
            )
            .unwrap();

        assert!(
            content.contains(
                "name,age",
            )
        );

        assert!(
            content.contains(
                "Alice,20",
            )
        );

        assert!(
            content.contains(
                "Bob,30",
            )
        );

        let _ =
            fs::remove_file(
                path,
            );
    }

    #[test]
    fn preserves_null_as_empty_csv_cell() {
        let dataframe =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "value".into(),
                        [
                            Some(10_i64),
                            None,
                            Some(30_i64),
                        ],
                    ),
                ],
            )
            .unwrap();

        let path =
            temporary_path();

        export_cleaned_csv(
            &dataframe,
            &path,
        )
        .unwrap();

        let content =
            fs::read_to_string(
                &path,
            )
            .unwrap();

        assert!(
            content.contains(
                "10"
            )
        );

        assert!(
            content.contains(
                "30"
            )
        );

        let _ =
            fs::remove_file(
                path,
            );
    }
}