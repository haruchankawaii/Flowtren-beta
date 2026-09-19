use polars::prelude::*;

use crate::column_profile::{
    profile_column,
    BasicColumnProfile,
};

#[derive(Debug, Clone)]
pub struct BasicDatasetProfile {
    pub row_count: usize,
    pub column_count: usize,
    pub columns: Vec<BasicColumnProfile>,
}

pub fn profile_dataframe(
    df: &DataFrame,
) -> PolarsResult<BasicDatasetProfile> {
    let mut columns =
        Vec::with_capacity(df.width());

    for column in df.columns() {
        let series =
            column.as_materialized_series();

        columns.push(
            profile_column(series)?
        );
    }

    Ok(BasicDatasetProfile {
        row_count: df.height(),
        column_count: df.width(),
        columns,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowtren_core::semantic_type::SemanticType;
    use std::path::PathBuf;

    #[test]
    fn profiles_basic_csv() {
        let path =
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR"),
            )
            .join(
                "../../tests/fixtures/csv/basic.csv",
            );

        let df =
            flowtren_io::csv::reader::read_csv(
                &path,
            )
            .expect("CSV should load");

        let profile =
            profile_dataframe(&df)
                .expect(
                    "Profiling should succeed",
                );

        assert_eq!(
            profile.row_count,
            3
        );

        assert_eq!(
            profile.column_count,
            3
        );

        assert_eq!(
            profile.columns[0].name,
            "date"
        );

        assert_eq!(
            profile.columns[0].null_count,
            0
        );

        assert_eq!(
            profile.columns[1].name,
            "region"
        );

        assert_eq!(
            profile.columns[1].unique_count,
            2
        );

        assert_eq!(
            profile.columns[2].name,
            "revenue"
        );

        assert_eq!(
            profile.columns[2].unique_count,
            3
        );

        assert_eq!(
            profile.columns[2]
                .semantic_type,
            SemanticType::Integer
        );

        assert_eq!(
            profile.columns[2]
                .semantic_confidence,
            1.0
        );
    }

    #[test]
    fn detects_category_column() {
        let path =
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR"),
            )
            .join(
                "../../tests/fixtures/csv/categories.csv",
            );

        let df =
            flowtren_io::csv::reader::read_csv(
                &path,
            )
            .expect("CSV should load");

        let profile =
            profile_dataframe(&df)
                .expect(
                    "Profiling should succeed",
                );

        assert_eq!(
            profile.row_count,
            10
        );

        assert_eq!(
            profile.column_count,
            2
        );

        assert_eq!(
            profile.columns[0]
                .semantic_type,
            SemanticType::Category
        );

        assert_eq!(
            profile.columns[1]
                .semantic_type,
            SemanticType::Integer
        );

        let confidence =
            profile.columns[0]
                .semantic_confidence;

        assert!(
            (confidence - 0.80).abs()
                < 0.001
        );
    }

    #[test]
    fn detects_advanced_semantic_types() {
        let path =
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR"),
            )
            .join(
                "../../tests/fixtures/csv/semantic-types.csv",
            );

        let df =
            flowtren_io::csv::reader::read_csv(
                &path,
            )
            .expect("CSV should load");

        let profile =
            profile_dataframe(&df)
                .expect(
                    "Profiling should succeed",
                );

        assert_eq!(
            profile.columns[0]
                .semantic_type,
            SemanticType::Date
        );

        assert_eq!(
            profile.columns[1]
                .semantic_type,
            SemanticType::Currency
        );

        assert_eq!(
            profile.columns[2]
                .semantic_type,
            SemanticType::Percentage
        );

        assert_eq!(
            profile.columns[0]
                .semantic_confidence,
            1.0
        );

        assert_eq!(
            profile.columns[1]
                .semantic_confidence,
            1.0
        );

        assert_eq!(
            profile.columns[2]
                .semantic_confidence,
            1.0
        );
    }

    #[test]
    fn detects_identity_semantic_types() {
        let path =
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR"),
            )
            .join(
                "../../tests/fixtures/csv/identifiers.csv",
            );

        let df =
            flowtren_io::csv::reader::read_csv(
                &path,
            )
            .expect("CSV should load");

        let profile =
            profile_dataframe(&df)
                .expect(
                    "Profiling should succeed",
                );

        assert_eq!(
            profile.columns[0]
                .semantic_type,
            SemanticType::Identifier
        );

        assert_eq!(
            profile.columns[1]
                .semantic_type,
            SemanticType::Email
        );

        assert_eq!(
            profile.columns[2]
                .semantic_type,
            SemanticType::Phone
        );

        assert_eq!(
            profile.columns[3]
                .semantic_type,
            SemanticType::Text
        );

        assert_eq!(
            profile.columns[0]
                .semantic_confidence,
            1.0
        );

        assert_eq!(
            profile.columns[1]
                .semantic_confidence,
            1.0
        );

        assert_eq!(
            profile.columns[2]
                .semantic_confidence,
            1.0
        );

        assert_eq!(
            profile.columns[3]
                .semantic_confidence,
            1.0
        );
    }

    #[test]
    fn semantic_confidence_is_valid() {
        let path =
            PathBuf::from(
                env!("CARGO_MANIFEST_DIR"),
            )
            .join(
                "../../tests/fixtures/csv/semantic-types.csv",
            );

        let df =
            flowtren_io::csv::reader::read_csv(
                &path,
            )
            .expect("CSV should load");

        let profile =
            profile_dataframe(&df)
                .expect(
                    "Profiling should succeed",
                );

        for column in profile.columns {
            assert!(
                column.semantic_confidence
                    >= 0.0
            );

            assert!(
                column.semantic_confidence
                    <= 1.0
            );
        }
    }
}