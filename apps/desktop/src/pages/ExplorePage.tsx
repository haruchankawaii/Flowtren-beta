import {
    useEffect,
    useMemo,
    useRef,
    useState,
  } from "react";
  
  import type {
    EChartsOption,
  } from "echarts";
  
  import {
    EChart,
  } from "../components/charts/EChart";
  
  import {
    useDataset,
  } from "../features/dataset/context";
  
  import {
    analyzeStatistics,
    getChartData,
    getCompatibleCharts,
    recommendCharts,
  } from "../features/explore/api";
  
  import type {
    ChartData,
    ChartRecommendation,
    NumericColumnStats,
    StatisticsResponse,
  } from "../features/explore/types";
  
  export function ExplorePage() {
    const {
      dataset,
    } =
      useDataset();
  
    const [
      statistics,
      setStatistics,
    ] =
      useState<
        StatisticsResponse | null
      >(
        null,
      );
  
    const [
      recommendations,
      setRecommendations,
    ] =
      useState<
        ChartRecommendation[]
      >(
        [],
      );
  
    const [
      selectedRecommendation,
      setSelectedRecommendation,
    ] =
      useState<
        ChartRecommendation | null
      >(
        null,
      );
  
    const [
      selectedChartType,
      setSelectedChartType,
    ] =
      useState(
        "",
      );
  
    const [
      compatibleCharts,
      setCompatibleCharts,
    ] =
      useState<string[]>(
        [],
      );
  
    const [
      chartData,
      setChartData,
    ] =
      useState<
        ChartData | null
      >(
        null,
      );
  
    const [
      selectedNumericColumn,
      setSelectedNumericColumn,
    ] =
      useState(
        "",
      );
  
    const [
      loading,
      setLoading,
    ] =
      useState(
        false,
      );
  
    const [
      chartLoading,
      setChartLoading,
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
  
    const chartDataCache =
      useRef<
        Map<string, ChartData>
      >(
        new Map(),
      );
  
    const compatibilityCache =
      useRef<
        Map<string, string[]>
      >(
        new Map(),
      );
  
    useEffect(
      () => {
        chartDataCache
          .current
          .clear();
  
        compatibilityCache
          .current
          .clear();
  
        setChartData(
          null,
        );
  
        setCompatibleCharts(
          [],
        );
  
        if (!dataset) {
          setStatistics(
            null,
          );
  
          setRecommendations(
            [],
          );
  
          setSelectedRecommendation(
            null,
          );
  
          setSelectedChartType(
            "",
          );
  
          setSelectedNumericColumn(
            "",
          );
  
          setError(
            null,
          );
  
          return;
        }
  
        void loadExplore();
      },
      [
        dataset,
      ],
    );
  
    async function loadExplore() {
      try {
        setLoading(
          true,
        );
  
        setError(
          null,
        );
  
        const [
          statisticsResult,
          chartResult,
        ] =
          await Promise.all([
            analyzeStatistics(),
            recommendCharts(),
          ]);
  
        setStatistics(
          statisticsResult,
        );
  
        setRecommendations(
          chartResult
            .recommendations,
        );
  
        if (
          statisticsResult
            .columns
            .length > 0
        ) {
          setSelectedNumericColumn(
            statisticsResult
              .columns[0]
              .column,
          );
        } else {
          setSelectedNumericColumn(
            "",
          );
        }
  
        const first =
          chartResult
            .recommendations[0]
            ?? null;
  
        setSelectedRecommendation(
          first,
        );
  
        if (first) {
          setSelectedChartType(
            first.chartType,
          );
        } else {
          setSelectedChartType(
            "",
          );
  
          setCompatibleCharts(
            [],
          );
  
          setChartData(
            null,
          );
        }
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
    }
  
    useEffect(
      () => {
        if (
          !selectedRecommendation
        ) {
          setCompatibleCharts(
            [],
          );
  
          return;
        }
  
        let cancelled =
          false;
  
        const recommendation =
          selectedRecommendation;
  
        const cacheKey =
          makeCompatibilityCacheKey(
            recommendation,
          );
  
        const cached =
          compatibilityCache
            .current
            .get(
              cacheKey,
            );
  
        if (cached) {
          setCompatibleCharts(
            cached,
          );
  
          return;
        }
  
        void (
          async () => {
            try {
              const result =
                await getCompatibleCharts(
                  recommendation
                    .xColumn,
  
                  recommendation
                    .yColumn,
                );
  
              if (cancelled) {
                return;
              }
  
              compatibilityCache
                .current
                .set(
                  cacheKey,
  
                  result
                    .chartTypes,
                );
  
              setCompatibleCharts(
                result
                  .chartTypes,
              );
            } catch (error) {
              if (cancelled) {
                return;
              }
  
              setError(
                normalizeError(
                  error,
                ),
              );
            }
          }
        )();
  
        return () => {
          cancelled =
            true;
        };
      },
      [
        selectedRecommendation,
      ],
    );
  
    useEffect(
      () => {
        if (
          !selectedRecommendation
          || !selectedChartType
        ) {
          setChartData(
            null,
          );
  
          setChartLoading(
            false,
          );
  
          return;
        }
  
        let cancelled =
          false;
  
        const recommendation =
          selectedRecommendation;
  
        const chartType =
          selectedChartType;
  
        const cacheKey =
          makeChartDataCacheKey(
            chartType,
            recommendation,
          );
  
        const cached =
          chartDataCache
            .current
            .get(
              cacheKey,
            );
  
        if (cached) {
          setChartData(
            cached,
          );
  
          setChartLoading(
            false,
          );
  
          return;
        }
  
        setChartLoading(
          true,
        );
  
        setChartData(
          null,
        );
  
        void (
          async () => {
            try {
              const data =
                await getChartData(
                  chartType,
  
                  recommendation
                    .xColumn,
  
                  recommendation
                    .yColumn,
                );
  
              if (cancelled) {
                return;
              }
  
              chartDataCache
                .current
                .set(
                  cacheKey,
                  data,
                );
  
              setChartData(
                data,
              );
            } catch (error) {
              if (cancelled) {
                return;
              }
  
              setError(
                normalizeError(
                  error,
                ),
              );
  
              setChartData(
                null,
              );
            } finally {
              if (!cancelled) {
                setChartLoading(
                  false,
                );
              }
            }
          }
        )();
  
        return () => {
          cancelled =
            true;
        };
      },
      [
        selectedRecommendation,
        selectedChartType,
      ],
    );
  
    const selectedStats =
      useMemo(
        () => {
          return statistics
            ?.columns
            .find(
              (
                column,
              ) =>
                column.column
                === selectedNumericColumn,
            )
            ?? null;
        },
        [
          statistics,
          selectedNumericColumn,
        ],
      );
  
    const chartOption =
      useMemo(
        () =>
          chartData
            ? buildChartOption(
                chartData,
              )
            : null,
        [
          chartData,
        ],
      );
  
    if (!dataset) {
      return (
        <section className="page">
          <div className="empty-state">
            <p className="eyebrow">
              Explore
            </p>
  
            <h1>
              No dataset loaded
            </h1>
  
            <p>
              Open a spreadsheet
              before exploring it.
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
              Explore
            </p>
  
            <h1>
              Explore your data
            </h1>
  
            <p className="page-description">
              Inspect descriptive
              statistics and visualize
              chart recommendations
              generated locally by
              Flowtren.
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
                chartDataCache
                  .current
                  .clear();
  
                compatibilityCache
                  .current
                  .clear();
  
                setChartData(
                  null,
                );
  
                setCompatibleCharts(
                  [],
                );
  
                void loadExplore();
              }
            }
          >
            {loading
              ? "Analyzing..."
              : "Refresh"}
          </button>
        </header>
  
        {error && (
          <div className="error-card">
            {error}
          </div>
        )}
  
        {statistics && (
          <>
            <div className="explore-summary-grid">
              <article className="summary-card">
                <span className="summary-label">
                  Rows
                </span>
  
                <strong>
                  {statistics
                    .rowCount
                    .toLocaleString()}
                </strong>
              </article>
  
              <article className="summary-card">
                <span className="summary-label">
                  Columns
                </span>
  
                <strong>
                  {
                    statistics
                      .columnCount
                  }
                </strong>
              </article>
  
              <article className="summary-card">
                <span className="summary-label">
                  Numeric columns
                </span>
  
                <strong>
                  {
                    statistics
                      .numericColumnCount
                  }
                </strong>
              </article>
  
              <article className="summary-card">
                <span className="summary-label">
                  Chart suggestions
                </span>
  
                <strong>
                  {
                    recommendations
                      .length
                  }
                </strong>
              </article>
            </div>
  
            <StatisticsPanel
              statistics={
                statistics
              }
              selectedColumn={
                selectedNumericColumn
              }
              selectedStats={
                selectedStats
              }
              onSelectColumn={
                setSelectedNumericColumn
              }
            />
          </>
        )}
  
        <div className="explore-layout">
          <aside className="content-card chart-recommendation-panel">
            <div className="card-heading">
              <p className="eyebrow">
                Recommended
              </p>
  
              <h2>
                Charts
              </h2>
            </div>
  
            {recommendations
              .length === 0 ? (
              <div className="explore-empty-small">
                No compatible charts
                found.
              </div>
            ) : (
              <div className="chart-recommendation-list">
                {recommendations.map(
                  (
                    recommendation,
                    index,
                  ) => {
                    const selected =
                      recommendation
                      === selectedRecommendation;
  
                    return (
                      <button
                        key={
                          [
                            recommendation
                              .chartType,
  
                            recommendation
                              .xColumn,
  
                            recommendation
                              .yColumn,
  
                            index,
                          ].join(
                            ":",
                          )
                        }
                        type="button"
                        className={
                          selected
                            ? "chart-recommendation-item chart-recommendation-item-active"
                            : "chart-recommendation-item"
                        }
                        onClick={
                          () => {
                            setSelectedRecommendation(
                              recommendation,
                            );
  
                            setSelectedChartType(
                              recommendation
                                .chartType,
                            );
  
                            setError(
                              null,
                            );
                          }
                        }
                      >
                        <div>
                          <strong>
                            {formatLabel(
                              recommendation
                                .chartType,
                            )}
                          </strong>
  
                          <span>
                            {recommendation
                              .xColumn}
  
                            {recommendation
                              .yColumn
                              ? ` → ${recommendation.yColumn}`
                              : ""}
                          </span>
                        </div>
  
                        <span className="chart-confidence">
                          {
                            recommendation
                              .confidence
                          }
                        </span>
                      </button>
                    );
                  },
                )}
              </div>
            )}
          </aside>
  
          <div className="content-card chart-view-card">
            <div className="chart-view-header">
              <div>
                <p className="eyebrow">
                  Visualization
                </p>
  
                <h2>
                  {selectedRecommendation
                    ? selectedRecommendation
                        .reason
                    : "Select a chart"}
                </h2>
              </div>
  
              {compatibleCharts
                .length > 1 && (
                <select
                  className="chart-type-select"
                  value={
                    selectedChartType
                  }
                  onChange={
                    (
                      event,
                    ) => {
                      setSelectedChartType(
                        event
                          .target
                          .value,
                      );
  
                      setError(
                        null,
                      );
                    }
                  }
                >
                  {compatibleCharts.map(
                    (
                      chartType,
                    ) => (
                      <option
                        key={
                          chartType
                        }
                        value={
                          chartType
                        }
                      >
                        {formatLabel(
                          chartType,
                        )}
                      </option>
                    ),
                  )}
                </select>
              )}
            </div>
  
            {chartLoading && (
              <div className="chart-loading">
                Building chart...
              </div>
            )}
  
            {!chartLoading
              && chartOption && (
              <EChart
                option={
                  chartOption
                }
              />
            )}
  
            {!chartLoading
              && !chartOption && (
              <div className="explore-empty-chart">
                Select a chart
                recommendation to
                visualize your data.
              </div>
            )}
          </div>
        </div>
      </section>
    );
  }
  
  type StatisticsPanelProps = {
    statistics:
      StatisticsResponse;
  
    selectedColumn:
      string;
  
    selectedStats:
      NumericColumnStats | null;
  
    onSelectColumn:
      (
        column:
          string,
      ) => void;
  };
  
  function StatisticsPanel(
    {
      statistics,
      selectedColumn,
      selectedStats,
      onSelectColumn,
    }: StatisticsPanelProps,
  ) {
    if (
      statistics.columns.length
      === 0
    ) {
      return null;
    }
  
    return (
      <div className="content-card statistics-panel">
        <div className="statistics-header">
          <div>
            <p className="eyebrow">
              Statistics
            </p>
  
            <h2>
              Numeric summary
            </h2>
          </div>
  
          <select
            className="chart-type-select"
            value={
              selectedColumn
            }
            onChange={
              (
                event,
              ) => {
                onSelectColumn(
                  event
                    .target
                    .value,
                );
              }
            }
          >
            {statistics
              .columns
              .map(
                (
                  column,
                ) => (
                  <option
                    key={
                      column.column
                    }
                    value={
                      column.column
                    }
                  >
                    {
                      column.column
                    }
                  </option>
                ),
              )}
          </select>
        </div>
  
        {selectedStats && (
          <div className="statistics-grid">
            <Stat
              label="Mean"
              value={
                selectedStats
                  .descriptive
                  .mean
              }
            />
  
            <Stat
              label="Median"
              value={
                selectedStats
                  .percentiles
                  .median
              }
            />
  
            <Stat
              label="Minimum"
              value={
                selectedStats
                  .descriptive
                  .min
              }
            />
  
            <Stat
              label="Maximum"
              value={
                selectedStats
                  .descriptive
                  .max
              }
            />
  
            <Stat
              label="Std. deviation"
              value={
                selectedStats
                  .descriptive
                  .stdDev
              }
            />
  
            <Stat
              label="P25"
              value={
                selectedStats
                  .percentiles
                  .p25
              }
            />
  
            <Stat
              label="P75"
              value={
                selectedStats
                  .percentiles
                  .p75
              }
            />
  
            <Stat
              label="Trend"
              value={
                selectedStats
                  .trend
                  .direction
                ?? "—"
              }
            />
          </div>
        )}
      </div>
    );
  }
  
  function Stat(
    {
      label,
      value,
    }: {
      label:
        string;
  
      value:
        number
        | string
        | null;
    },
  ) {
    return (
      <div className="stat-item">
        <span>
          {label}
        </span>
  
        <strong>
          {typeof value
            === "number"
            ? formatNumber(
                value,
              )
            : value
              ?? "—"}
        </strong>
      </div>
    );
  }
  
  function buildChartOption(
    data:
      ChartData,
  ): EChartsOption {
    const base:
      EChartsOption = {
      animationDuration:
        350,
  
      tooltip: {
        trigger:
          data.chartType
            === "pie"
            ? "item"
            : "axis",
      },
  
      grid: {
        left: 55,
        right: 24,
        top: 30,
        bottom: 70,
      },
    };
  
    switch (
      data.chartType
    ) {
      case "histogram":
        return {
          ...base,
  
          xAxis: {
            type:
              "category",
  
            data:
              data.bins.map(
                (
                  bin,
                ) =>
                  bin.label,
              ),
  
            axisLabel: {
              rotate: 30,
            },
          },
  
          yAxis: {
            type:
              "value",
  
            name:
              "Count",
          },
  
          series: [
            {
              type:
                "bar",
  
              data:
                data.bins.map(
                  (
                    bin,
                  ) =>
                    bin.count,
                ),
            },
          ],
        };
  
      case "scatter":
        return {
          ...base,
  
          tooltip: {
            trigger:
              "item",
          },
  
          xAxis: {
            type:
              "value",
  
            name:
              data.xColumn,
          },
  
          yAxis: {
            type:
              "value",
  
            name:
              data.yColumn
              ?? "",
          },
  
          series: [
            {
              type:
                "scatter",
  
              symbolSize:
                8,
  
              data:
                data.points.map(
                  (
                    point,
                  ) => [
                    point.x,
                    point.y,
                  ],
                ),
            },
          ],
        };
  
      case "line":
        return {
          ...base,
  
          xAxis: {
            type:
              "category",
  
            data:
              data.labels,
  
            axisLabel: {
              hideOverlap:
                true,
            },
          },
  
          yAxis: {
            type:
              "value",
          },
  
          series: [
            {
              type:
                "line",
  
              data:
                data.values,
  
              showSymbol:
                data.values.length
                < 80,
  
              smooth:
                false,
            },
          ],
        };
  
      case "pie":
        return {
          tooltip: {
            trigger:
              "item",
          },
  
          legend: {
            bottom: 0,
          },
  
          series: [
            {
              type:
                "pie",
  
              radius: [
                "42%",
                "68%",
              ],
  
              data:
                data.labels.map(
                  (
                    label,
                    index,
                  ) => ({
                    name:
                      label,
  
                    value:
                      data.values[
                        index
                      ],
                  }),
                ),
            },
          ],
        };
  
      case "bar":
      default:
        return {
          ...base,
  
          xAxis: {
            type:
              "category",
  
            data:
              data.labels,
  
            axisLabel: {
              rotate:
                data.labels.length
                > 8
                  ? 30
                  : 0,
            },
          },
  
          yAxis: {
            type:
              "value",
          },
  
          series: [
            {
              type:
                "bar",
  
              data:
                data.values,
            },
          ],
        };
    }
  }
  
  function makeCompatibilityCacheKey(
    recommendation:
      ChartRecommendation,
  ): string {
    return [
      recommendation
        .xColumn,
  
      recommendation
        .yColumn
        ?? "",
    ].join(
      "::",
    );
  }
  
  function makeChartDataCacheKey(
    chartType:
      string,
  
    recommendation:
      ChartRecommendation,
  ): string {
    return [
      chartType,
  
      recommendation
        .xColumn,
  
      recommendation
        .yColumn
        ?? "",
    ].join(
      "::",
    );
  }
  
  function formatNumber(
    value:
      number,
  ): string {
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
  
  function formatLabel(
    value:
      string,
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