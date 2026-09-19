export type InsightSeverity =
  | "info"
  | "low"
  | "medium"
  | "high"
  | string;

export type InsightConfidence =
  | "low"
  | "medium"
  | "high"
  | string;

export type InsightEvidence = {
  label: string;

  value:
    number
    | string;
};

export type Insight = {
  kind: string;

  title: string;

  summary: string;

  columns: string[];

  severity:
    InsightSeverity;

  confidence:
    InsightConfidence;

  priorityScore: number;

  evidence:
    InsightEvidence[];
};

export type InsightsResponse = {
  insightCount: number;

  insights: Insight[];
};