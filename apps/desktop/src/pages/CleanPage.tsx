import {
  useEffect,
  useMemo,
  useState,
} from "react";

import {
  applyCleaning,
  getQualityComparison,
} from "../features/dataset/api";

import {
  useDataset,
} from "../features/dataset/context";

import type {
  CleaningSuggestion,
  QualityComparison,
} from "../features/dataset/types";

export function CleanPage() {
  const {
    dataset,
    refreshDataset,
  } =
    useDataset();

  const [
    selectedIds,
    setSelectedIds,
  ] =
    useState<
      Set<number>
    >(
      new Set(),
    );

  const [
    applying,
    setApplying,
  ] =
    useState(
      false,
    );

  const [
    localError,
    setLocalError,
  ] =
    useState<
      string | null
    >(
      null,
    );

  const [
    comparison,
    setComparison,
  ] =
    useState<
      QualityComparison | null
    >(
      null,
    );

  useEffect(
    () => {
      if (!dataset) {
        setSelectedIds(
          new Set(),
        );

        return;
      }

      setSelectedIds(
        new Set(
          dataset
            .cleaningSuggestions
            .map(
              (
                suggestion,
              ) =>
                suggestion.id,
            ),
        ),
      );
    },
    [
      dataset
        ?.cleaningSuggestions,
    ],
  );

  useEffect(
    () => {
      if (dataset) {
        return;
      }

      setComparison(
        null,
      );

      setLocalError(
        null,
      );

      setApplying(
        false,
      );
    },
    [
      dataset,
    ],
  );

  const suggestions =
    dataset
      ?.cleaningSuggestions
    ?? [];

  const allSelected =
    suggestions.length
    > 0
    && selectedIds.size
      === suggestions.length;

  const selectedAffectedRows =
    useMemo(
      () => {
        return suggestions
          .filter(
            (
              suggestion,
            ) =>
              selectedIds.has(
                suggestion.id,
              ),
          )
          .reduce(
            (
              total,
              suggestion,
            ) =>
              total
              + suggestion
                  .affectedRows,

            0,
          );
      },
      [
        suggestions,
        selectedIds,
      ],
    );

  if (!dataset) {
    return (
      <section className="page">
        <div className="empty-state">
          <p className="eyebrow">
            Clean
          </p>

          <h1>
            No dataset loaded
          </h1>

          <p>
            Open a spreadsheet from
            Home before cleaning it.
          </p>
        </div>
      </section>
    );
  }

  function toggleSuggestion(
    id: number,
  ) {
    setSelectedIds(
      (
        current,
      ) => {
        const next =
          new Set(
            current,
          );

        if (
          next.has(
            id,
          )
        ) {
          next.delete(
            id,
          );
        } else {
          next.add(
            id,
          );
        }

        return next;
      },
    );
  }

  function toggleAll() {
    if (allSelected) {
      setSelectedIds(
        new Set(),
      );

      return;
    }

    setSelectedIds(
      new Set(
        suggestions.map(
          (
            suggestion,
          ) =>
            suggestion.id,
        ),
      ),
    );
  }

  async function handleApply() {
    if (
      selectedIds.size
      === 0
    ) {
      return;
    }

    try {
      setApplying(
        true,
      );

      setLocalError(
        null,
      );

      const ids =
        Array.from(
          selectedIds,
        );

      await applyCleaning(
        ids,
      );

      await refreshDataset();

      const qualityComparison =
        await getQualityComparison();

      setComparison(
        qualityComparison,
      );

      setSelectedIds(
        new Set(),
      );
    } catch (error) {
      setLocalError(
        normalizeError(
          error,
        ),
      );
    } finally {
      setApplying(
        false,
      );
    }
  }

  return (
    <section className="page">
      <header className="page-header">
        <div>
          <p className="eyebrow">
            Clean
          </p>

          <h1>
            Clean your data
          </h1>

          <p className="page-description">
            Flowtren found
            deterministic cleaning
            opportunities without
            changing your original
            spreadsheet.
          </p>
        </div>
      </header>

      <div className="clean-summary-grid">
        <article className="summary-card">
          <span className="summary-label">
            Current quality
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
            Suggestions
          </span>

          <strong>
            {
              suggestions.length
            }
          </strong>
        </article>

        <article className="summary-card">
          <span className="summary-label">
            Selected
          </span>

          <strong>
            {
              selectedIds.size
            }
          </strong>
        </article>

        <article className="summary-card">
          <span className="summary-label">
            Rows affected
          </span>

          <strong>
            {selectedAffectedRows
              .toLocaleString()}
          </strong>
        </article>
      </div>

      {comparison && (
        <div className="quality-comparison-card">
          <div>
            <span className="summary-label">
              Quality improvement
            </span>

            <strong>
              {comparison
                .beforeScore
                .toFixed(0)}
              {" → "}
              {comparison
                .afterScore
                .toFixed(0)}
            </strong>
          </div>

          <div>
            <span className="summary-label">
              Resolved issues
            </span>

            <strong>
              {
                comparison
                  .resolvedIssueCount
              }
            </strong>
          </div>

          <div>
            <span className="summary-label">
              Remaining issues
            </span>

            <strong>
              {
                comparison
                  .remainingIssueCount
              }
            </strong>
          </div>
        </div>
      )}

      <div className="content-card">
        <div className="clean-toolbar">
          <div>
            <p className="eyebrow">
              Suggestions
            </p>

            <h2>
              Recommended changes
            </h2>
          </div>

          {suggestions.length
            > 0 && (
            <button
              type="button"
              className="secondary-button"
              onClick={
                toggleAll
              }
            >
              {allSelected
                ? "Clear selection"
                : "Select all"}
            </button>
          )}
        </div>

        {suggestions.length
          === 0 ? (
          <div className="clean-empty">
            <div className="clean-empty-icon">
              ✓
            </div>

            <h3>
              Nothing to clean
            </h3>

            <p>
              Flowtren does not
              currently see any safe
              cleaning suggestions.
            </p>
          </div>
        ) : (
          <div className="suggestion-list">
            {suggestions.map(
              (
                suggestion,
              ) => (
                <SuggestionRow
                  key={
                    suggestion.id
                  }
                  suggestion={
                    suggestion
                  }
                  selected={
                    selectedIds.has(
                      suggestion.id,
                    )
                  }
                  onToggle={
                    () =>
                      toggleSuggestion(
                        suggestion.id,
                      )
                  }
                />
              ),
            )}
          </div>
        )}
      </div>

      {localError && (
        <div className="error-card">
          {localError}
        </div>
      )}

      {suggestions.length
        > 0 && (
        <div className="clean-action-bar">
          <div>
            <strong>
              {
                selectedIds.size
              }
              {" "}
              selected
            </strong>

            <span>
              Changes apply only to
              Flowtren&apos;s working copy.
            </span>
          </div>

          <button
            type="button"
            className="primary-button"
            disabled={
              applying
              || selectedIds.size
              === 0
            }
            onClick={
              () => {
                void handleApply();
              }
            }
          >
            {applying
              ? "Applying..."
              : `Apply ${selectedIds.size} changes`}
          </button>
        </div>
      )}
    </section>
  );
}

type SuggestionRowProps = {
  suggestion:
    CleaningSuggestion;

  selected:
    boolean;

  onToggle:
    () => void;
};

function SuggestionRow(
  {
    suggestion,
    selected,
    onToggle,
  }: SuggestionRowProps,
) {
  return (
    <label className="suggestion-row">
      <input
        type="checkbox"
        checked={
          selected
        }
        onChange={
          onToggle
        }
      />

      <div className="suggestion-main">
        <div className="suggestion-title-row">
          <strong>
            {formatCleaningKind(
              suggestion.kind,
            )}
          </strong>

          <span
            className={
              confidenceClass(
                suggestion.confidence,
              )
            }
          >
            {formatConfidence(
              suggestion.confidence,
            )}
          </span>
        </div>

        <div className="suggestion-meta">
          <span>
            {suggestion.column
              ?? "Entire dataset"}
          </span>

          <span>
            •
          </span>

          <span>
            {suggestion
              .affectedRows
              .toLocaleString()}
            {" "}
            rows affected
          </span>
        </div>
      </div>
    </label>
  );
}

function formatCleaningKind(
  value: string,
): string {
  return value
    .replace(
      /([a-z])([A-Z])/g,
      "$1 $2",
    )
    .replace(
      /_/g,
      " ",
    )
    .replace(
      /\b\w/g,
      (
        character,
      ) =>
        character
          .toUpperCase(),
    );
}

function formatConfidence(
  confidence: number,
): string {
  if (
    confidence >= 0.9
  ) {
    return "High confidence";
  }

  if (
    confidence >= 0.7
  ) {
    return "Medium confidence";
  }

  return "Low confidence";
}

function confidenceClass(
  confidence: number,
): string {
  if (
    confidence >= 0.9
  ) {
    return "confidence-badge confidence-high";
  }

  if (
    confidence >= 0.7
  ) {
    return "confidence-badge confidence-medium";
  }

  return "confidence-badge confidence-low";
}

function normalizeError(
  error: unknown,
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