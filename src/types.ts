/* Shared types aligned with Rust commands */
export type Decision = "keep" | "review" | "discard";

/* Tea variety model aligned with Rust */
export type TeaVariety = {
  id: string;
  name: string;
  generation: string;
  location: string;
  year: number;
  germination_rate: number;
  growth_score: number;
  disease_resistance: number;
  aroma_score: number;
};

/* Analysis model aligned with Rust */
export type AnalysisResult = {
  id: string;
  total_score: number;
  decision: Decision;
};

/* Analysis config for scoring */
export type AnalysisWeights = {
  germination: number;
  growth: number;
  disease: number;
  aroma: number;
};

export type AnalysisConfig = {
  weights: AnalysisWeights;
  keep_threshold: number;
  review_threshold: number;
  normalize_to_100: boolean;
};

/* Merged row for UI */
export type Row = TeaVariety & AnalysisResult;

/* Export row for CSV saving */
export type ExportRow = Row;

