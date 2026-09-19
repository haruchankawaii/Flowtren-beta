import {
    useState,
  } from "react";
  
  import {
    useDataset,
  } from "../features/dataset/context";
  
  import {
    exportWithDialog,
  } from "../features/export/api";
  
  import type {
    ExportFormat,
    ExportResponse,
    ExportState,
  } from "../features/export/types";
  
  export function ExportPage() {
    const {
      dataset,
    } =
      useDataset();
  
    const [
      selectedFormat,
      setSelectedFormat,
    ] =
      useState<ExportFormat>(
        "xlsx",
      );
  
    const [
      state,
      setState,
    ] =
      useState<ExportState>(
        "idle",
      );
  
    const [
      result,
      setResult,
    ] =
      useState<
        ExportResponse | null
      >(
        null,
      );
  
    const [
      error,
      setError,
    ] =
      useState<
        string | null
      >(
        null,
      );
  
    if (!dataset) {
      return (
        <section className="page">
          <div className="empty-state">
            <p className="eyebrow">
              Export
            </p>
  
            <h1>
              No dataset loaded
            </h1>
  
            <p>
              Open a spreadsheet
              before exporting data.
            </p>
          </div>
        </section>
      );
    }
  
    async function handleExport() {
        if (!dataset) {
          return;
        }
      
        const fileName =
          dataset.fileName;
      
        try {
          setError(
            null,
          );
      
          setResult(
            null,
          );
      
          setState(
            "exporting",
          );
      
          const exported =
            await exportWithDialog(
              selectedFormat,
              fileName,
            );
      
          if (!exported) {
            setState(
              "idle",
            );
      
            return;
          }
      
          setResult(
            exported,
          );
      
          setState(
            "success",
          );
        } catch (error) {
          setError(
            normalizeError(
              error,
            ),
          );
      
          setState(
            "error",
          );
        }
      }
  
    return (
      <section className="page">
        <header className="page-header">
          <div>
            <p className="eyebrow">
              Export
            </p>
  
            <h1>
              Export your cleaned data
            </h1>
  
            <p className="page-description">
              Save Flowtren&apos;s
              current working copy as
              CSV or Excel without
              modifying your original
              spreadsheet.
            </p>
          </div>
        </header>
  
        <div className="export-summary-grid">
          <article className="summary-card">
            <span className="summary-label">
              Rows
            </span>
  
            <strong>
              {dataset
                .profile
                .rowCount
                .toLocaleString()}
            </strong>
          </article>
  
          <article className="summary-card">
            <span className="summary-label">
              Columns
            </span>
  
            <strong>
              {dataset
                .profile
                .columnCount
                .toLocaleString()}
            </strong>
          </article>
  
          <article className="summary-card">
            <span className="summary-label">
              Quality
            </span>
  
            <strong>
              {dataset
                .quality
                .score
                .toFixed(0)}
              /100
            </strong>
          </article>
  
          <article className="summary-card">
            <span className="summary-label">
              Remaining suggestions
            </span>
  
            <strong>
              {
                dataset
                  .cleaningSuggestions
                  .length
              }
            </strong>
          </article>
        </div>
  
        <div className="content-card export-card">
          <div className="card-heading">
            <p className="eyebrow">
              Format
            </p>
  
            <h2>
              Choose export format
            </h2>
          </div>
  
          <div className="export-format-grid">
            <FormatCard
              format="xlsx"
              title="Excel workbook"
              extension=".xlsx"
              description="Best for sharing, reporting, and opening again in Excel or compatible spreadsheet apps."
              selected={
                selectedFormat
                === "xlsx"
              }
              onSelect={
                setSelectedFormat
              }
            />
  
            <FormatCard
              format="csv"
              title="CSV file"
              extension=".csv"
              description="Simple, portable tabular data suitable for analysis tools, scripts, and databases."
              selected={
                selectedFormat
                === "csv"
              }
              onSelect={
                setSelectedFormat
              }
            />
          </div>
  
          <div className="export-details">
            <div className="export-detail-row">
              <span>
                Source
              </span>
  
              <strong>
                {
                  dataset.fileName
                }
              </strong>
            </div>
  
            <div className="export-detail-row">
              <span>
                Suggested output
              </span>
  
              <strong>
                {suggestedOutputName(
                  dataset.fileName,
                  selectedFormat,
                )}
              </strong>
            </div>
  
            <div className="export-detail-row">
              <span>
                Original file
              </span>
  
              <strong className="export-safe-value">
                Never modified
              </strong>
            </div>
  
            <div className="export-detail-row">
              <span>
                Processing
              </span>
  
              <strong className="export-safe-value">
                Local only
              </strong>
            </div>
          </div>
  
          <div className="export-actions">
            <div>
              <strong>
                Ready to export
              </strong>
  
              <span>
                You will choose the
                destination using your
                system&apos;s Save dialog.
              </span>
            </div>
  
            <button
              type="button"
              className="primary-button"
              disabled={
                state
                === "exporting"
              }
              onClick={
                () => {
                  void handleExport();
                }
              }
            >
              {state
                === "exporting"
                ? "Exporting..."
                : selectedFormat
                  === "xlsx"
                  ? "Export Excel"
                  : "Export CSV"}
            </button>
          </div>
        </div>
  
        {state
          === "success"
          && result && (
          <ExportSuccess
            result={
              result
            }
          />
        )}
  
        {state
          === "error"
          && error && (
          <div className="error-card">
            {error}
          </div>
        )}
  
        <div className="export-privacy-card">
          <div className="export-privacy-icon">
            ✓
          </div>
  
          <div>
            <strong>
              Your data stays on
              this computer
            </strong>
  
            <p>
              Flowtren writes the
              exported file directly
              to the location you
              choose. Spreadsheet data
              is not uploaded.
            </p>
          </div>
        </div>
      </section>
    );
  }
  
  type FormatCardProps = {
    format:
      ExportFormat;
  
    title:
      string;
  
    extension:
      string;
  
    description:
      string;
  
    selected:
      boolean;
  
    onSelect:
      (
        format:
          ExportFormat,
      ) => void;
  };
  
  function FormatCard(
    {
      format,
      title,
      extension,
      description,
      selected,
      onSelect,
    }: FormatCardProps,
  ) {
    return (
      <button
        type="button"
        className={
          selected
            ? "export-format-card export-format-card-selected"
            : "export-format-card"
        }
        onClick={
          () => {
            onSelect(
              format,
            );
          }
        }
      >
        <div className="export-format-header">
          <div className="export-format-icon">
            {format
              === "xlsx"
              ? "X"
              : "C"}
          </div>
  
          <div>
            <strong>
              {title}
            </strong>
  
            <span>
              {extension}
            </span>
          </div>
  
          <div
            className={
              selected
                ? "export-radio export-radio-selected"
                : "export-radio"
            }
          >
            {selected
              ? "✓"
              : ""}
          </div>
        </div>
  
        <p>
          {description}
        </p>
      </button>
    );
  }
  
  function ExportSuccess(
    {
      result,
    }: {
      result:
        ExportResponse;
    },
  ) {
    return (
      <div className="export-success-card">
        <div className="export-success-icon">
          ✓
        </div>
  
        <div className="export-success-content">
          <div>
            <p className="eyebrow">
              Export complete
            </p>
  
            <h2>
              Your file is ready
            </h2>
          </div>
  
          <div className="export-result-grid">
            <div>
              <span>
                Format
              </span>
  
              <strong>
                {result
                  .format
                  .toUpperCase()}
              </strong>
            </div>
  
            <div>
              <span>
                Rows
              </span>
  
              <strong>
                {result
                  .rowCount
                  .toLocaleString()}
              </strong>
            </div>
  
            <div>
              <span>
                Columns
              </span>
  
              <strong>
                {
                  result
                    .columnCount
                }
              </strong>
            </div>
  
            <div>
              <span>
                Quality
              </span>
  
              <strong>
                {result
                  .qualityScore
                  !== null
                  ? `${result.qualityScore.toFixed(0)}/100`
                  : "—"}
              </strong>
            </div>
          </div>
  
          <div className="export-output-path">
            <span>
              Saved to
            </span>
  
            <strong>
              {
                result
                  .outputPath
              }
            </strong>
          </div>
        </div>
      </div>
    );
  }
  
  function suggestedOutputName(
    sourceFileName:
      string,
  
    format:
      ExportFormat,
  ): string {
    const lastDot =
      sourceFileName
        .lastIndexOf(
          ".",
        );
  
    const baseName =
      lastDot > 0
        ? sourceFileName
            .slice(
              0,
              lastDot,
            )
        : sourceFileName;
  
    return `${baseName}_cleaned.${format}`;
  }
  
  function normalizeError(
    error:
      unknown,
  ): string {
    if (
      error instanceof Error
    ) {
      return error.message;
    }
  
    return String(
      error,
    );
  }