use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use polars::prelude::DataFrame;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum ExportFormat {
    Csv,
    Xlsx,
}

impl ExportFormat {
    pub fn as_str(
        &self,
    ) -> &'static str {
        match self {
            ExportFormat::Csv => {
                "csv"
            }

            ExportFormat::Xlsx => {
                "xlsx"
            }
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
)]
pub struct ExportMetadata {
    pub source_file:
        Option<String>,

    pub output_file:
        String,

    pub format:
        ExportFormat,

    pub row_count:
        usize,

    pub column_count:
        usize,

    pub cleaning_operations:
        usize,

    pub quality_score:
        Option<f64>,

    pub exported_at_unix_seconds:
        u64,
}

impl ExportMetadata {
    pub fn new(
        output_file:
            impl Into<String>,

        format:
            ExportFormat,

        row_count:
            usize,

        column_count:
            usize,
    ) -> Self {
        Self {
            source_file:
                None,

            output_file:
                output_file.into(),

            format,

            row_count,

            column_count,

            cleaning_operations:
                0,

            quality_score:
                None,

            exported_at_unix_seconds:
                current_unix_seconds(),
        }
    }

    pub fn from_dataframe(
        dataframe: &DataFrame,

        output_file:
            impl Into<String>,

        format:
            ExportFormat,
    ) -> Self {
        Self::new(
            output_file,

            format,

            dataframe.height(),

            dataframe.width(),
        )
    }

    pub fn with_source_file(
        mut self,
        source_file:
            impl Into<String>,
    ) -> Self {
        self.source_file =
            Some(
                source_file.into(),
            );

        self
    }

    pub fn with_cleaning_operations(
        mut self,
        cleaning_operations:
            usize,
    ) -> Self {
        self.cleaning_operations =
            cleaning_operations;

        self
    }

    pub fn with_quality_score(
        mut self,
        quality_score:
            f64,
    ) -> Self {
        self.quality_score =
            Some(
                quality_score.clamp(
                    0.0,
                    100.0,
                ),
            );

        self
    }
}

fn current_unix_seconds()
    -> u64
{
    SystemTime::now()
        .duration_since(
            UNIX_EPOCH,
        )
        .map(
            |duration| {
                duration.as_secs()
            },
        )
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    use polars::prelude::*;

    #[test]
    fn creates_metadata_from_dataframe() {
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

        let metadata =
            ExportMetadata::from_dataframe(
                &dataframe,

                "cleaned.csv",

                ExportFormat::Csv,
            );

        assert_eq!(
            metadata.row_count,
            2
        );

        assert_eq!(
            metadata.column_count,
            2
        );

        assert_eq!(
            metadata.format,
            ExportFormat::Csv
        );

        assert_eq!(
            metadata.output_file,
            "cleaned.csv"
        );
    }

    #[test]
    fn adds_optional_metadata() {
        let metadata =
            ExportMetadata::new(
                "cleaned.xlsx",
                ExportFormat::Xlsx,
                100,
                5,
            )
            .with_source_file(
                "raw.xlsx",
            )
            .with_cleaning_operations(
                12,
            )
            .with_quality_score(
                91.5,
            );

        assert_eq!(
            metadata.source_file,
            Some(
                "raw.xlsx".to_string()
            )
        );

        assert_eq!(
            metadata.cleaning_operations,
            12
        );

        assert_eq!(
            metadata.quality_score,
            Some(91.5)
        );
    }

    #[test]
    fn quality_score_is_clamped() {
        let metadata =
            ExportMetadata::new(
                "test.csv",
                ExportFormat::Csv,
                1,
                1,
            )
            .with_quality_score(
                150.0,
            );

        assert_eq!(
            metadata.quality_score,
            Some(100.0)
        );
    }

    #[test]
    fn format_has_stable_string() {
        assert_eq!(
            ExportFormat::Csv
                .as_str(),
            "csv"
        );

        assert_eq!(
            ExportFormat::Xlsx
                .as_str(),
            "xlsx"
        );
    }
}