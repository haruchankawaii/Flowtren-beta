import {
  createContext,
  useContext,
  useMemo,
  useState,
} from "react";

import type {
  ReactNode,
} from "react";

import {
  buildDatasetSummary,
  chooseSpreadsheet,
  openDataset,
  refreshDatasetAnalysis,
} from "./api";

import type {
  DatasetAnalysisStatus,
  DatasetSummary,
  OpeningDataset,
} from "./types";

type DatasetContextValue = {
  dataset:
    DatasetSummary | null;

  openingDataset:
    OpeningDataset | null;

  loading:
    boolean;

  analysisStatus:
    DatasetAnalysisStatus;

  error:
    string | null;

  analysisError:
    string | null;

  openSpreadsheet:
    () => Promise<void>;

  refreshDataset:
    () => Promise<void>;

  clearError:
    () => void;
};

const DatasetContext =
  createContext<
    DatasetContextValue
    | undefined
  >(
    undefined,
  );

type DatasetProviderProps = {
  children:
    ReactNode;
};

export function DatasetProvider(
  {
    children,
  }: DatasetProviderProps,
) {
  const [
    dataset,
    setDataset,
  ] =
    useState<
      DatasetSummary | null
    >(
      null,
    );

  const [
    openingDataset,
    setOpeningDataset,
  ] =
    useState<
      OpeningDataset | null
    >(
      null,
    );

  const [
    loading,
    setLoading,
  ] =
    useState(
      false,
    );

  const [
    analysisStatus,
    setAnalysisStatus,
  ] =
    useState<
      DatasetAnalysisStatus
    >(
      "idle",
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

  const [
    analysisError,
    setAnalysisError,
  ] =
    useState<
      string | null
    >(
      null,
    );

  async function openSpreadsheet() {
    try {
      setError(
        null,
      );

      setAnalysisError(
        null,
      );

      const path =
        await chooseSpreadsheet();

      if (!path) {
        return;
      }

      /*
       * loading hanya mewakili proses
       * membuka / membaca file.
       *
       * Setelah open_dataset selesai,
       * loading dimatikan sehingga Home
       * bisa langsung menampilkan metadata.
       */
      setLoading(
        true,
      );

      setAnalysisStatus(
        "idle",
      );

      const opened =
        await openDataset(
          path,
        );

      /*
       * Dataset Rust sekarang sudah loaded.
       *
       * Tampilkan metadata ke UI sebelum
       * profile / quality / cleaning selesai.
       */
      setOpeningDataset(
        opened,
      );

      /*
       * Dataset sebelumnya tidak boleh tetap
       * dianggap aktif karena backend Rust
       * sudah diganti dengan dataset baru.
       */
      setDataset(
        null,
      );

      setLoading(
        false,
      );

      setAnalysisStatus(
        "analyzing",
      );

      try {
        const result =
          await buildDatasetSummary(
            opened,
          );

        setDataset(
          result,
        );

        setOpeningDataset(
          null,
        );

        setAnalysisStatus(
          "ready",
        );
      } catch (
        analysisFailure
      ) {
        setAnalysisError(
          normalizeError(
            analysisFailure,
          ),
        );

        setAnalysisStatus(
          "error",
        );
      }
    } catch (openFailure) {
      setError(
        normalizeError(
          openFailure,
        ),
      );

      setAnalysisStatus(
        "idle",
      );
    } finally {
      setLoading(
        false,
      );
    }
  }

  async function refreshDataset() {
    if (!dataset) {
      return;
    }

    try {
      setError(
        null,
      );

      setAnalysisError(
        null,
      );

      setAnalysisStatus(
        "analyzing",
      );

      const analysis =
        await refreshDatasetAnalysis();

      setDataset(
        (current) => {
          if (!current) {
            return null;
          }

          return {
            ...current,

            profile:
              analysis.profile,

            quality:
              analysis.quality,

            cleaningSuggestions:
              analysis
                .cleaningSuggestions,
          };
        },
      );

      setAnalysisStatus(
        "ready",
      );
    } catch (
      refreshFailure
    ) {
      const message =
        normalizeError(
          refreshFailure,
        );

      setError(
        message,
      );

      setAnalysisError(
        message,
      );

      setAnalysisStatus(
        "error",
      );

      throw refreshFailure;
    }
  }

  function clearError() {
    setError(
      null,
    );

    setAnalysisError(
      null,
    );
  }

  const value =
    useMemo(
      () => ({
        dataset,
        openingDataset,
        loading,
        analysisStatus,
        error,
        analysisError,
        openSpreadsheet,
        refreshDataset,
        clearError,
      }),
      [
        dataset,
        openingDataset,
        loading,
        analysisStatus,
        error,
        analysisError,
      ],
    );

  return (
    <DatasetContext.Provider
      value={
        value
      }
    >
      {children}
    </DatasetContext.Provider>
  );
}

export function useDataset():
  DatasetContextValue
{
  const context =
    useContext(
      DatasetContext,
    );

  if (!context) {
    throw new Error(
      "useDataset must be used inside DatasetProvider",
    );
  }

  return context;
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