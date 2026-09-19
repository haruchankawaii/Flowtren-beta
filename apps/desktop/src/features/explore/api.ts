import {
    invoke,
  } from "@tauri-apps/api/core";
  
  import type {
    ChartData,
    ChartRecommendationsResponse,
    CompatibleChartsResponse,
    StatisticsResponse,
  } from "./types";
  
  export async function analyzeStatistics():
    Promise<StatisticsResponse>
  {
    return invoke<StatisticsResponse>(
      "analyze_statistics",
    );
  }
  
  export async function recommendCharts():
    Promise<ChartRecommendationsResponse>
  {
    return invoke<ChartRecommendationsResponse>(
      "recommend_charts",
    );
  }
  
  export async function getCompatibleCharts(
    xColumn: string,
  
    yColumn:
      string | null,
  ): Promise<
    CompatibleChartsResponse
  > {
    return invoke<CompatibleChartsResponse>(
      "get_compatible_charts",
      {
        xColumn,
        yColumn,
      },
    );
  }
  
  export async function getChartData(
    chartType: string,
  
    xColumn: string,
  
    yColumn:
      string | null,
  ): Promise<ChartData> {
    return invoke<ChartData>(
      "get_chart_data",
      {
        chartType,
        xColumn,
        yColumn,
      },
    );
  }