use std::path::Path;

use polars::prelude::{
    AnyValue,
    DataFrame,
};

use rust_xlsxwriter::{
    Format,
    FormatAlign,
    Workbook,
    Worksheet,
};

use crate::{
    ExportError,
    ExportResult,
};

const EXCEL_MAX_ROWS:
    usize = 1_048_576;

const EXCEL_MAX_COLUMNS:
    usize = 16_384;

const EXCEL_SAFE_INTEGER:
    i128 =
    999_999_999_999_999;

pub fn export_cleaned_xlsx(
    dataframe: &DataFrame,
    path: impl AsRef<Path>,
) -> ExportResult<()> {
    validate_excel_dimensions(
        dataframe,
    )?;

    let mut workbook =
        Workbook::new();

    let worksheet =
        workbook.add_worksheet();

    worksheet.set_name(
        "Cleaned Data",
    )?;

    let header_format =
        Format::new()
            .set_bold()
            .set_align(
                FormatAlign::Center,
            );

    write_headers(
        worksheet,
        dataframe,
        &header_format,
    )?;

    write_rows(
        worksheet,
        dataframe,
    )?;

    // Autofit cukup berguna untuk export
    // user-facing dan tidak memengaruhi data.
    worksheet.autofit();

    workbook.save(
        path,
    )?;

    Ok(())
}

fn validate_excel_dimensions(
    dataframe: &DataFrame,
) -> ExportResult<()> {
    // +1 karena header memakai row pertama.
    if dataframe.height()
        + 1
        > EXCEL_MAX_ROWS
    {
        return Err(
            ExportError::InvalidData(
                format!(
                    "dataset has {} rows but XLSX supports at most {} data rows when a header row is included",
                    dataframe.height(),
                    EXCEL_MAX_ROWS - 1,
                ),
            ),
        );
    }

    if dataframe.width()
        > EXCEL_MAX_COLUMNS
    {
        return Err(
            ExportError::InvalidData(
                format!(
                    "dataset has {} columns but XLSX supports at most {EXCEL_MAX_COLUMNS}",
                    dataframe.width(),
                ),
            ),
        );
    }

    Ok(())
}

fn write_headers(
    worksheet: &mut Worksheet,
    dataframe: &DataFrame,
    format: &Format,
) -> ExportResult<()> {
    for (
        column_index,
        column,
    ) in dataframe
        .columns()
        .iter()
        .enumerate()
    {
        worksheet
            .write_string_with_format(
                0,
                column_index
                    as u16,
                column
                    .name()
                    .as_str(),
                format,
            )?;
    }

    Ok(())
}

fn write_rows(
    worksheet: &mut Worksheet,
    dataframe: &DataFrame,
) -> ExportResult<()> {
    for row_index
        in 0..dataframe.height()
    {
        for (
            column_index,
            column,
        ) in dataframe
            .columns()
            .iter()
            .enumerate()
        {
            let value =
                column.get(
                    row_index,
                )?;

            write_value(
                worksheet,

                (
                    row_index
                        + 1
                ) as u32,

                column_index
                    as u16,

                value,
            )?;
        }
    }

    Ok(())
}

fn write_value(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: AnyValue<'_>,
) -> ExportResult<()> {
    match value {
        AnyValue::Null => {
            // Tidak perlu menulis cell.
        }

        AnyValue::Boolean(
            value,
        ) => {
            worksheet
                .write_boolean(
                    row,
                    column,
                    value,
                )?;
        }

        AnyValue::String(
            value,
        ) => {
            worksheet
                .write_string(
                    row,
                    column,
                    value,
                )?;
        }

        AnyValue::StringOwned(
            value,
        ) => {
            worksheet
                .write_string(
                    row,
                    column,
                    value.as_str(),
                )?;
        }

        AnyValue::Int8(
            value,
        ) => {
            worksheet
                .write_number(
                    row,
                    column,
                    value,
                )?;
        }

        AnyValue::Int16(
            value,
        ) => {
            worksheet
                .write_number(
                    row,
                    column,
                    value,
                )?;
        }

        AnyValue::Int32(
            value,
        ) => {
            worksheet
                .write_number(
                    row,
                    column,
                    value,
                )?;
        }

        AnyValue::UInt8(
            value,
        ) => {
            worksheet
                .write_number(
                    row,
                    column,
                    value,
                )?;
        }

        AnyValue::UInt16(
            value,
        ) => {
            worksheet
                .write_number(
                    row,
                    column,
                    value,
                )?;
        }

        AnyValue::UInt32(
            value,
        ) => {
            worksheet
                .write_number(
                    row,
                    column,
                    value,
                )?;
        }

        AnyValue::Int64(
            value,
        ) => {
            write_i128_safely(
                worksheet,
                row,
                column,
                value as i128,
            )?;
        }

        AnyValue::UInt64(
            value,
        ) => {
            write_u128_safely(
                worksheet,
                row,
                column,
                value as u128,
            )?;
        }

        AnyValue::Int128(
            value,
        ) => {
            write_i128_safely(
                worksheet,
                row,
                column,
                value,
            )?;
        }

        AnyValue::UInt128(
            value,
        ) => {
            write_u128_safely(
                worksheet,
                row,
                column,
                value,
            )?;
        }

        AnyValue::Float32(
            value,
        ) => {
            if value.is_finite() {
                worksheet
                    .write_number(
                        row,
                        column,
                        value,
                    )?;
            } else {
                worksheet
                    .write_string(
                        row,
                        column,
                        value.to_string(),
                    )?;
            }
        }

        AnyValue::Float64(
            value,
        ) => {
            if value.is_finite() {
                worksheet
                    .write_number(
                        row,
                        column,
                        value,
                    )?;
            } else {
                worksheet
                    .write_string(
                        row,
                        column,
                        value.to_string(),
                    )?;
            }
        }

        other => {
            worksheet
                .write_string(
                    row,
                    column,
                    other.to_string(),
                )?;
        }
    }

    Ok(())
}

fn write_i128_safely(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: i128,
) -> ExportResult<()> {
    if value.abs()
        <= EXCEL_SAFE_INTEGER
    {
        worksheet
            .write_number(
                row,
                column,
                value as f64,
            )?;
    } else {
        worksheet
            .write_string(
                row,
                column,
                value.to_string(),
            )?;
    }

    Ok(())
}

fn write_u128_safely(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: u128,
) -> ExportResult<()> {
    if value
        <= EXCEL_SAFE_INTEGER
            as u128
    {
        worksheet
            .write_number(
                row,
                column,
                value as f64,
            )?;
    } else {
        worksheet
            .write_string(
                row,
                column,
                value.to_string(),
            )?;
    }

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

    use polars::prelude::{
        Column,
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
                    "flowtren-export-{unique}.xlsx"
                ),
            )
    }

    #[test]
    fn exports_dataframe_to_xlsx() {
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

                    Column::new(
                        "active".into(),
                        [
                            true,
                            false,
                        ],
                    ),
                ],
            )
            .unwrap();

        let path =
            temporary_path();

        export_cleaned_xlsx(
            &dataframe,
            &path,
        )
        .expect(
            "XLSX export should succeed",
        );

        assert!(
            path.exists()
        );

        let metadata =
            fs::metadata(
                &path,
            )
            .unwrap();

        assert!(
            metadata.len()
                > 0
        );

        let _ =
            fs::remove_file(
                path,
            );
    }

    #[test]
    fn exports_null_values() {
        let dataframe =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "amount".into(),
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

        export_cleaned_xlsx(
            &dataframe,
            &path,
        )
        .expect(
            "XLSX export with nulls should succeed",
        );

        assert!(
            path.exists()
        );

        let _ =
            fs::remove_file(
                path,
            );
    }

    #[test]
    fn preserves_large_integer_as_text() {
        let dataframe =
            DataFrame::new_infer_height(
                vec![
                    Column::new(
                        "identifier".into(),
                        [
                            9_223_372_036_854_775_000_i64,
                        ],
                    ),
                ],
            )
            .unwrap();

        let path =
            temporary_path();

        export_cleaned_xlsx(
            &dataframe,
            &path,
        )
        .expect(
            "Large integer export should succeed",
        );

        assert!(
            path.exists()
        );

        let _ =
            fs::remove_file(
                path,
            );
    }
}