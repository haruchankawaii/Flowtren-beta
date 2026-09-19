import {
    invoke,
  } from "@tauri-apps/api/core";
  
  import type {
    Insight,
    InsightsResponse,
  } from "./types";
  
  type BackendInsightsResponse =
    | InsightsResponse
    | Insight[];
  
  export async function generateInsights():
    Promise<InsightsResponse>
  {
    const response =
      await invoke<BackendInsightsResponse>(
        "generate_insights",
      );
  
    if (
      Array.isArray(
        response,
      )
    ) {
      const insights =
        sortInsights(
          response,
        );
  
      return {
        insightCount:
          insights.length,
  
        insights,
      };
    }
  
    const insights =
      sortInsights(
        response.insights
          ?? [],
      );
  
    return {
      insightCount:
        response.insightCount
        ?? insights.length,
  
      insights,
    };
  }
  
  function sortInsights(
    insights: Insight[],
  ): Insight[] {
    return [
      ...insights,
    ].sort(
      (
        left,
        right,
      ) =>
        right.priorityScore
        - left.priorityScore,
    );
  }