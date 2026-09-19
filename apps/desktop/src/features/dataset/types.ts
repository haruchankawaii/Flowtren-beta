export type ColumnProfile = {
  name: string;

  dtype: string;

  semanticType: string;

  semanticConfidence: number;

  nullCount: number;

  uniqueCount: number;
};

export type DatasetProfile = {
  rowCount: number;

  columnCount: number;

  columns: ColumnProfile[];
};

export type DatasetQuality = {
  rowCount: number;

  columnCount: number;

  duplicateCount: number;

  duplicateRate: number;

  score: number;
};

export type CleaningSuggestion = {
  id: number;

  column: string | null;

  kind: string;

  affectedRows: number;

  confidence: number;
};

export type DatasetSummary = {
  path: string;

  fileName: string;

  profile: DatasetProfile;

  quality: DatasetQuality;

  cleaningSuggestions:
    CleaningSuggestion[];
};

export type OpeningDataset = {
  path: string;

  fileName: string;

  fileType: string;

  rowCount: number;

  columnCount: number;
};

export type DatasetAnalysisStatus =
  | "idle"
  | "analyzing"
  | "ready"
  | "error";

export type QualityComparison = {
  beforeScore: number;

  afterScore: number;

  scoreChange: number;

  improved: boolean;

  beforeIssueCount: number;

  afterIssueCount: number;

  resolvedIssueCount: number;

  newIssueCount: number;

  remainingIssueCount: number;
};

export type ApplyCleaningResponse = {
  appliedSuggestionCount?: number;

  totalChanges?: number;
};