import {
    invoke,
  } from "@tauri-apps/api/core";
  
  import {
    save,
  } from "@tauri-apps/plugin-dialog";
  
  import type {
    ExportFormat,
    ExportResponse,
  } from "./types";
  
  export async function chooseExportPath(
    format:
      ExportFormat,
  
    suggestedFileName:
      string,
  ): Promise<string | null> {
    const extension =
      format === "csv"
        ? "csv"
        : "xlsx";
  
    const selected =
      await save({
        defaultPath:
          ensureExtension(
            suggestedFileName,
            extension,
          ),
  
        filters: [
          {
            name:
              format === "csv"
                ? "CSV file"
                : "Excel workbook",
  
            extensions: [
              extension,
            ],
          },
        ],
      });
  
    return selected
      ?? null;
  }
  
  export async function exportDataset(
    format:
      ExportFormat,
  
    outputPath:
      string,
  
    overwrite = false,
  ): Promise<ExportResponse> {
    const command =
      format === "csv"
        ? "export_dataset_csv"
        : "export_dataset_xlsx";
  
    return invoke<ExportResponse>(
      command,
      {
        outputPath,
        overwrite,
      },
    );
  }
  
  export async function exportWithDialog(
    format:
      ExportFormat,
  
    sourceFileName:
      string,
  ): Promise<
    ExportResponse | null
  > {
    const outputPath =
      await chooseExportPath(
        format,
        buildSuggestedFileName(
          sourceFileName,
          format,
        ),
      );
  
    if (!outputPath) {
      return null;
    }
  
    return exportDataset(
      format,
      outputPath,
      false,
    );
  }
  
  function buildSuggestedFileName(
    sourceFileName:
      string,
  
    format:
      ExportFormat,
  ): string {
    const extension =
      format === "csv"
        ? "csv"
        : "xlsx";
  
    const baseName =
      removeExtension(
        sourceFileName,
      );
  
    return `${baseName}_cleaned.${extension}`;
  }
  
  function removeExtension(
    fileName:
      string,
  ): string {
    const lastDot =
      fileName.lastIndexOf(
        ".",
      );
  
    if (
      lastDot <= 0
    ) {
      return fileName;
    }
  
    return fileName.slice(
      0,
      lastDot,
    );
  }
  
  function ensureExtension(
    fileName:
      string,
  
    extension:
      string,
  ): string {
    const expected =
      `.${extension}`;
  
    if (
      fileName
        .toLowerCase()
        .endsWith(
          expected,
        )
    ) {
      return fileName;
    }
  
    return `${fileName}${expected}`;
  }