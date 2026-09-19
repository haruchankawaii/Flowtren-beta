use std::path::Path;

use calamine::{
    open_workbook,
    Data,
    Range,
    Reader,
    Xlsx,
    XlsxError,
};
use polars::prelude::*;

use crate::error::IoError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetInfo {
    pub name: String,

    // Jumlah row data, tidak termasuk header.
    pub row_count: usize,

    pub column_count: usize,

    pub is_empty: bool,
}

pub fn list_xlsx_sheets(
    path: &Path,
) -> Result<Vec<String>, IoError> {
    if !path.exists() {
        return Err(IoError::FileNotFound);
    }

    let workbook: Xlsx<_> =
        open_workbook(path)
            .map_err(|err: XlsxError| {
                IoError::Xlsx(err.to_string())
            })?;

    Ok(workbook.sheet_names().to_vec())
}

pub fn inspect_xlsx_sheets(
    path: &Path,
) -> Result<Vec<SheetInfo>, IoError> {
    if !path.exists() {
        return Err(IoError::FileNotFound);
    }

    let mut workbook: Xlsx<_> =
        open_workbook(path)
            .map_err(|err: XlsxError| {
                IoError::Xlsx(err.to_string())
            })?;

    let sheet_names =
        workbook.sheet_names().to_vec();

    let mut result =
        Vec::with_capacity(sheet_names.len());

    for sheet_name in sheet_names {
        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|err| {
                IoError::Xlsx(err.to_string())
            })?;

        result.push(
            inspect_sheet_range(
                &sheet_name,
                &range,
            )
        );
    }

    Ok(result)
}

pub fn read_xlsx_first_sheet(
    path: &Path,
) -> Result<DataFrame, IoError> {
    if !path.exists() {
        return Err(IoError::FileNotFound);
    }

    let mut workbook: Xlsx<_> =
        open_workbook(path)
            .map_err(|err: XlsxError| {
                IoError::Xlsx(err.to_string())
            })?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| {
            IoError::Xlsx(
                "Workbook has no sheets".to_string(),
            )
        })?;

    read_sheet_from_workbook(
        &mut workbook,
        &sheet_name,
    )
}

pub fn read_xlsx_sheet_by_name(
    path: &Path,
    sheet_name: &str,
) -> Result<DataFrame, IoError> {
    if !path.exists() {
        return Err(IoError::FileNotFound);
    }

    let mut workbook: Xlsx<_> =
        open_workbook(path)
            .map_err(|err: XlsxError| {
                IoError::Xlsx(err.to_string())
            })?;

    let exists = workbook
        .sheet_names()
        .iter()
        .any(|name| name == sheet_name);

    if !exists {
        return Err(
            IoError::Xlsx(
                format!(
                    "Sheet '{sheet_name}' not found"
                ),
            ),
        );
    }

    read_sheet_from_workbook(
        &mut workbook,
        sheet_name,
    )
}

fn read_sheet_from_workbook<RS>(
    workbook: &mut Xlsx<RS>,
    sheet_name: &str,
) -> Result<DataFrame, IoError>
where
    RS: std::io::Read
        + std::io::Seek,
{
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|err| {
            IoError::Xlsx(err.to_string())
        })?;

    range_to_dataframe(&range)
}

fn inspect_sheet_range(
    name: &str,
    range: &Range<Data>,
) -> SheetInfo {
    if range.is_empty() {
        return SheetInfo {
            name: name.to_string(),
            row_count: 0,
            column_count: 0,
            is_empty: true,
        };
    }

    let total_rows = range.height();
    let column_count = range.width();

    let row_count =
        total_rows.saturating_sub(1);

    SheetInfo {
        name: name.to_string(),
        row_count,
        column_count,
        is_empty: false,
    }
}

fn range_to_dataframe(
    range: &Range<Data>,
) -> Result<DataFrame, IoError> {
    let mut rows = range.rows();

    let header_row = rows
        .next()
        .ok_or_else(|| {
            IoError::Xlsx(
                "Sheet is empty".to_string(),
            )
        })?;

    let headers: Vec<String> = header_row
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            let value =
                cell_to_string(cell);

            let trimmed =
                value.trim();

            if trimmed.is_empty() {
                format!(
                    "column_{}",
                    index + 1
                )
            } else {
                trimmed.to_string()
            }
        })
        .collect();

    if headers.is_empty() {
        return Err(
            IoError::Xlsx(
                "Sheet has no columns".to_string(),
            ),
        );
    }

    let mut columns:
        Vec<Vec<String>> =
        vec![Vec::new(); headers.len()];

    for row in rows {
        for column_index in 0..headers.len() {
            let value = row
                .get(column_index)
                .map(cell_to_string)
                .unwrap_or_default();

            columns[column_index]
                .push(value);
        }
    }

    let dataframe_columns:
        Vec<Column> = headers
        .iter()
        .zip(columns.iter())
        .map(|(name, values)| {
            Column::new(
                name.as_str().into(),
                values.clone(),
            )
        })
        .collect();

    DataFrame::new_infer_height(
        dataframe_columns,
    )
    .map_err(|err| {
        IoError::Xlsx(err.to_string())
    })
}

fn cell_to_string(
    cell: &Data,
) -> String {
    match cell {
        Data::Empty => {
            String::new()
        }

        Data::String(value) => {
            value.clone()
        }

        Data::Float(value) => {
            value.to_string()
        }

        Data::Int(value) => {
            value.to_string()
        }

        Data::Bool(value) => {
            value.to_string()
        }

        Data::DateTime(value) => {
            value.to_string()
        }

        Data::DateTimeIso(value) => {
            value.clone()
        }

        Data::DurationIso(value) => {
            value.clone()
        }

        Data::Error(value) => {
            format!("{value:?}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(
        filename: &str,
    ) -> PathBuf {
        PathBuf::from(
            env!("CARGO_MANIFEST_DIR"),
        )
        .join(
            "../../tests/fixtures/xlsx",
        )
        .join(filename)
    }

    #[test]
    fn reads_basic_xlsx() {
        let path =
            fixture_path("basic.xlsx");

        let df =
            read_xlsx_first_sheet(
                &path,
            )
            .expect(
                "XLSX should load",
            );

        assert_eq!(
            df.height(),
            3
        );

        assert_eq!(
            df.width(),
            3
        );

        assert_eq!(
            df.get_column_names(),
            &[
                "date",
                "region",
                "revenue",
            ]
        );
    }

    #[test]
    fn lists_xlsx_sheets() {
        let path =
            fixture_path(
                "multi-sheet.xlsx",
            );

        let sheets =
            list_xlsx_sheets(
                &path,
            )
            .expect(
                "Sheets should load",
            );

        assert_eq!(
            sheets,
            vec![
                "January",
                "February",
            ]
        );
    }

    #[test]
    fn inspects_xlsx_sheets() {
        let path =
            fixture_path(
                "multi-sheet.xlsx",
            );

        let sheets =
            inspect_xlsx_sheets(
                &path,
            )
            .expect(
                "Sheet inspection should succeed",
            );

        assert_eq!(
            sheets.len(),
            2
        );

        assert_eq!(
            sheets[0].name,
            "January"
        );

        assert_eq!(
            sheets[0].row_count,
            2
        );

        assert_eq!(
            sheets[0].column_count,
            2
        );

        assert_eq!(
            sheets[0].is_empty,
            false
        );

        assert_eq!(
            sheets[1].name,
            "February"
        );

        assert_eq!(
            sheets[1].row_count,
            2
        );

        assert_eq!(
            sheets[1].column_count,
            2
        );
    }

    #[test]
    fn reads_xlsx_sheet_by_name() {
        let path =
            fixture_path(
                "multi-sheet.xlsx",
            );

        let df =
            read_xlsx_sheet_by_name(
                &path,
                "February",
            )
            .expect(
                "February sheet should load",
            );

        assert_eq!(
            df.height(),
            2
        );

        assert_eq!(
            df.width(),
            2
        );

        assert_eq!(
            df.get_column_names(),
            &[
                "region",
                "revenue",
            ]
        );
    }

    #[test]
    fn returns_error_for_missing_sheet() {
        let path =
            fixture_path(
                "multi-sheet.xlsx",
            );

        let result =
            read_xlsx_sheet_by_name(
                &path,
                "DoesNotExist",
            );

        assert!(
            result.is_err()
        );
    }
}