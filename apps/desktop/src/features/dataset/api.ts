import {
  invoke,
} from "@tauri-apps/api/core";

import {
  open,
} from "@tauri-apps/plugin-dialog";

import type {
  ApplyCleaningResponse,
  CleaningSuggestion,
  DatasetProfile,
  DatasetQuality,
  DatasetSummary,
  OpeningDataset,
  QualityComparison,
} from "./types";

type OpenDatasetResponse = {
  fileName: string;

  filePath: string;

  fileType: string;

  rowCount: number;

  columnCount: number;
};

type CleaningSuggestionsResponse = {
  suggestions:
    CleaningSuggestion[];
};

export async function chooseSpreadsheet():
  Promise<string | null>
{
  const selected =
    await open({
      multiple: false,

      filters: [
        {
          name: "Spreadsheet",

          extensions: [
            "csv",
            "xlsx",
          ],
        },
      ],
    });

  if (!selected) {
    return null;
  }

  if (
    Array.isArray(
      selected,
    )
  ) {
    return selected[0]
      ?? null;
  }

  return selected;
}

export async function openDataset(
  path: string,
): Promise<OpeningDataset> {
  const response =
    await invoke<OpenDatasetResponse>(
      "open_dataset",
      {
        path,
      },
    );

  return {
    path:
      response.filePath,

    fileName:
      response.fileName,

    fileType:
      response.fileType,

    rowCount:
      response.rowCount,

    columnCount:
      response.columnCount,
  };
}

export async function getProfile():
  Promise<DatasetProfile>
{
  return invoke<DatasetProfile>(
    "profile_dataset",
  );
}

export async function getQuality():
  Promise<DatasetQuality>
{
  return invoke<DatasetQuality>(
    "analyze_quality",
  );
}

export async function getCleaningSuggestions():
  Promise<CleaningSuggestion[]>
{
  const response =
    await invoke<
      CleaningSuggestionsResponse
    >(
      "suggest_cleaning",
    );

  return response
    .suggestions;
}

export async function getQualityComparison():
  Promise<QualityComparison>
{
  return invoke<QualityComparison>(
    "compare_dataset_quality",
  );
}

export async function applyCleaning(
  suggestionIds?:
    number[],
): Promise<ApplyCleaningResponse> {
  return invoke<ApplyCleaningResponse>(
    "apply_cleaning",
    {
      suggestionIds:
        suggestionIds
        ?? null,
    },
  );
}

export async function refreshDatasetAnalysis():
  Promise<{
    profile: DatasetProfile;

    quality: DatasetQuality;

    cleaningSuggestions:
      CleaningSuggestion[];
  }>
{
  const [
    profile,
    quality,
    cleaningSuggestions,
  ] =
    await Promise.all([
      getProfile(),

      getQuality(),

      getCleaningSuggestions(),
    ]);

  return {
    profile,

    quality,

    cleaningSuggestions,
  };
}

export async function buildDatasetSummary(
  opened:
    OpeningDataset,
): Promise<DatasetSummary> {
  const analysis =
    await refreshDatasetAnalysis();

  return {
    path:
      opened.path,

    fileName:
      opened.fileName,

    profile:
      analysis.profile,

    quality:
      analysis.quality,

    cleaningSuggestions:
      analysis
        .cleaningSuggestions,
  };
}