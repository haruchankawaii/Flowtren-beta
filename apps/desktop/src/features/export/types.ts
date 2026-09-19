export type ExportFormat =
  | "csv"
  | "xlsx";

export type ExportResponse = {
  outputPath: string;

  format: ExportFormat;

  rowCount: number;

  columnCount: number;

  sourceFile:
    string | null;

  qualityScore:
    number | null;

  exportedAtUnixSeconds:
    number;
};

export type ExportState =
  | "idle"
  | "exporting"
  | "success"
  | "error";