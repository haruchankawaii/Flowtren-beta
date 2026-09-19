import {
    useCallback,
    useEffect,
    useMemo,
    useState,
  } from "react";
  
  import {
    useDataset,
  } from "../features/dataset/context";
  
  import {
    generateInsights,
  } from "../features/insights/api";
  
  import type {
    Insight,
  } from "../features/insights/types";
  
  export function InsightsPage() {
    const {
      dataset,
    } =
      useDataset();
  
    const [
      insights,
      setInsights,
    ] =
      useState<Insight[]>(
        [],
      );
  
    const [
      loading,
      setLoading,
    ] =
      useState(
        false,
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
  
    const loadInsights =
      useCallback(
        async () => {
          if (!dataset) {
            setInsights(
              [],
            );
  
            return;
          }
  
          try {
            setLoading(
              true,
            );
  
            setError(
              null,
            );
  
            const result =
              await generateInsights();
  
            setInsights(
              result.insights,
            );
          } catch (error) {
            setError(
              normalizeError(
                error,
              ),
            );
          } finally {
            setLoading(
              false,
            );
          }
        },
        [
          dataset,
        ],
      );
  
    useEffect(
      () => {
        void loadInsights();
      },
      [
        loadInsights,
      ],
    );
  
    const highPriorityCount =
      useMemo(
        () =>
          insights.filter(
            (
              insight,
            ) =>
              normalizeValue(
                insight.severity,
              )
              === "high",
          ).length,
        [
          insights,
        ],
      );
  
    const highConfidenceCount =
      useMemo(
        () =>
          insights.filter(
            (
              insight,
            ) =>
              normalizeValue(
                insight.confidence,
              )
              === "high",
          ).length,
        [
          insights,
        ],
      );
  
    if (!dataset) {
      return (
        <section className="page">
          <div className="empty-state">
            <p className="eyebrow">
              Insights
            </p>
  
            <h1>
              No dataset loaded
            </h1>
  
            <p>
              Open a spreadsheet from
              Home before generating
              insights.
            </p>
          </div>
        </section>
      );
    }
  
    return (
      <section className="page">
        <header className="page-header">
          <div>
            <p className="eyebrow">
              Insights
            </p>
  
            <h1>
              Understand your data
            </h1>
  
            <p className="page-description">
              Flowtren analyzes patterns,
              trends, anomalies, data
              quality, and relationships
              using deterministic local
              analysis.
            </p>
          </div>
  
          <button
            type="button"
            className="secondary-button"
            disabled={
              loading
            }
            onClick={
              () => {
                void loadInsights();
              }
            }
          >
            {loading
              ? "Analyzing..."
              : "Refresh insights"}
          </button>
        </header>
  
        <div className="insight-summary-grid">
          <article className="summary-card">
            <span className="summary-label">
              Insights found
            </span>
  
            <strong>
              {
                insights.length
              }
            </strong>
          </article>
  
          <article className="summary-card">
            <span className="summary-label">
              High severity
            </span>
  
            <strong>
              {
                highPriorityCount
              }
            </strong>
          </article>
  
          <article className="summary-card">
            <span className="summary-label">
              High confidence
            </span>
  
            <strong>
              {
                highConfidenceCount
              }
            </strong>
          </article>
  
          <article className="summary-card">
            <span className="summary-label">
              Dataset quality
            </span>
  
            <strong>
              {dataset
                .quality
                .score
                .toFixed(0)}
              /100
            </strong>
          </article>
        </div>
  
        {error && (
          <div className="error-card">
            {error}
          </div>
        )}
  
        {loading
          && insights.length
          === 0 && (
          <InsightsLoading />
        )}
  
        {!loading
          && insights.length
          === 0
          && !error && (
          <div className="content-card">
            <div className="insights-empty">
              <div className="insights-empty-icon">
                ✓
              </div>
  
              <h2>
                No notable insights
              </h2>
  
              <p>
                Flowtren did not find
                any patterns significant
                enough to highlight in
                this dataset.
              </p>
            </div>
          </div>
        )}
  
        {insights.length
          > 0 && (
          <div className="insight-list">
            {insights.map(
              (
                insight,
                index,
              ) => (
                <InsightCard
                  key={
                    makeInsightKey(
                      insight,
                      index,
                    )
                  }
                  insight={
                    insight
                  }
                />
              ),
            )}
          </div>
        )}
      </section>
    );
  }
  
  type InsightCardProps = {
    insight:
      Insight;
  };
  
  function InsightCard(
    {
      insight,
    }: InsightCardProps,
  ) {
    return (
      <article className="insight-card">
        <div className="insight-card-header">
          <div className="insight-title-area">
            <div className="insight-kind">
              {formatLabel(
                insight.kind,
              )}
            </div>
  
            <h2>
              {
                insight.title
              }
            </h2>
          </div>
  
          <div className="insight-badges">
            <span
              className={
                severityClass(
                  insight.severity,
                )
              }
            >
              {formatLabel(
                insight.severity,
              )}
              {" severity"}
            </span>
  
            <span
              className={
                confidenceClass(
                  insight.confidence,
                )
              }
            >
              {formatLabel(
                insight.confidence,
              )}
              {" confidence"}
            </span>
          </div>
        </div>
  
        <p className="insight-summary">
          {
            insight.summary
          }
        </p>
  
        <div className="insight-meta">
          {insight.columns.length
            > 0 && (
            <div className="insight-section">
              <span className="insight-section-label">
                Columns
              </span>
  
              <div className="column-chip-list">
                {insight.columns.map(
                  (
                    column,
                  ) => (
                    <span
                      key={
                        column
                      }
                      className="column-chip"
                    >
                      {column}
                    </span>
                  ),
                )}
              </div>
            </div>
          )}
  
          <div className="insight-priority">
            <span className="insight-section-label">
              Priority score
            </span>
  
            <strong>
              {formatNumber(
                insight.priorityScore,
              )}
            </strong>
          </div>
        </div>
  
        {insight.evidence.length
          > 0 && (
          <div className="evidence-section">
            <span className="insight-section-label">
              Evidence
            </span>
  
            <div className="evidence-grid">
              {insight.evidence.map(
                (
                  evidence,
                  index,
                ) => (
                  <div
                    key={
                      `${evidence.label}-${index}`
                    }
                    className="evidence-item"
                  >
                    <span>
                      {
                        formatLabel(
                          evidence.label,
                        )
                      }
                    </span>
  
                    <strong>
                      {
                        formatEvidenceValue(
                          evidence.value,
                        )
                      }
                    </strong>
                  </div>
                ),
              )}
            </div>
          </div>
        )}
      </article>
    );
  }
  
  function InsightsLoading() {
    return (
      <div className="insight-list">
        {[
          0,
          1,
          2,
        ].map(
          (
            item,
          ) => (
            <div
              key={
                item
              }
              className="insight-card insight-skeleton"
            >
              <div className="skeleton-line skeleton-small" />
  
              <div className="skeleton-line skeleton-title" />
  
              <div className="skeleton-line" />
  
              <div className="skeleton-line skeleton-medium" />
            </div>
          ),
        )}
      </div>
    );
  }
  
  function severityClass(
    value: string,
  ): string {
    switch (
      normalizeValue(
        value,
      )
    ) {
      case "high":
        return "insight-badge severity-high";
  
      case "medium":
        return "insight-badge severity-medium";
  
      case "low":
        return "insight-badge severity-low";
  
      default:
        return "insight-badge severity-info";
    }
  }
  
  function confidenceClass(
    value: string,
  ): string {
    switch (
      normalizeValue(
        value,
      )
    ) {
      case "high":
        return "insight-badge confidence-high";
  
      case "medium":
        return "insight-badge confidence-medium";
  
      default:
        return "insight-badge confidence-low";
    }
  }
  
  function normalizeValue(
    value: string,
  ): string {
    return value
      .trim()
      .toLowerCase();
  }
  
  function formatLabel(
    value: string,
  ): string {
    const spaced =
      value
        .replace(
          /([a-z])([A-Z])/g,
          "$1 $2",
        )
        .replace(
          /_/g,
          " ",
        )
        .trim();
  
    if (!spaced) {
      return "";
    }
  
    return spaced
      .split(
        /\s+/,
      )
      .map(
        (
          part,
        ) =>
          part
            .charAt(0)
            .toUpperCase()
          + part
            .slice(1)
            .toLowerCase(),
      )
      .join(
        " ",
      );
  }
  
  function formatNumber(
    value: number,
  ): string {
    if (
      !Number.isFinite(
        value,
      )
    ) {
      return String(
        value,
      );
    }
  
    return new Intl.NumberFormat(
      undefined,
      {
        maximumFractionDigits:
          3,
      },
    ).format(
      value,
    );
  }
  
  function formatEvidenceValue(
    value:
      number
      | string,
  ): string {
    if (
      typeof value
        === "number"
    ) {
      return formatNumber(
        value,
      );
    }
  
    return value;
  }
  
  function makeInsightKey(
    insight:
      Insight,
  
    index:
      number,
  ): string {
    return [
      insight.kind,
      insight.title,
      insight.columns.join(
        "-",
      ),
      index,
    ].join(
      "::",
    );
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