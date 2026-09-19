export type DescriptiveStats = {
    rowCount: number;
    count: number;
    nullCount: number;
  
    sum: number | null;
    mean: number | null;
    min: number | null;
    max: number | null;
    stdDev: number | null;
  };
  
  export type PercentileStats = {
    p25: number | null;
    median: number | null;
    p75: number | null;
  };
  
  export type TrendStats = {
    pairCount: number;
  
    slope: number | null;
  
    intercept: number | null;
  
    rSquared: number | null;
  
    direction: string | null;
  };
  
  export type NumericColumnStats = {
    column: string;
  
    descriptive:
      DescriptiveStats;
  
    percentiles:
      PercentileStats;
  
    trend:
      TrendStats;
  };
  
  export type Correlation = {
    leftColumn: string;
  
    rightColumn: string;
  
    pairCount: number;
  
    pearson: number | null;
  };
  
  export type StatisticsResponse = {
    rowCount: number;
  
    columnCount: number;
  
    numericColumnCount: number;
  
    columns:
      NumericColumnStats[];
  
    correlations:
      Correlation[];
  };
  
  export type ChartRecommendation = {
    chartType: string;
  
    xColumn: string;
  
    yColumn:
      string | null;
  
    confidence: string;
  
    reason: string;
  };
  
  export type ChartRecommendationsResponse = {
    recommendationCount: number;
  
    recommendations:
      ChartRecommendation[];
  };
  
  export type CompatibleChartsResponse = {
    xColumn: string;
  
    yColumn:
      string | null;
  
    chartTypes:
      string[];
  };
  
  export type ScatterPoint = {
    x: number;
    y: number;
  };
  
  export type HistogramBin = {
    label: string;
  
    start: number;
  
    end: number;
  
    count: number;
  };
  
  export type ChartData = {
    chartType: string;
  
    xColumn: string;
  
    yColumn:
      string | null;
  
    labels:
      string[];
  
    values:
      number[];
  
    points:
      ScatterPoint[];
  
    bins:
      HistogramBin[];
  };