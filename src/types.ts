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

/* Merged row for UI */
export type Row = TeaVariety & AnalysisResult;

/* Export row for CSV saving */
export type ExportRow = Row;

/* Sort key and direction (aligned with Rust SortKey/SortDir) */
export type SortKey =
  | "total_score"
  | "year"
  | "name"
  | "generation"
  | "decision"
  | "id";

export type SortDir = "asc" | "desc";

/* Summary view model (aligned with Rust) */
export type SummaryCounts = {
  keep: number;
  review: number;
  discard: number;
};

export type SummaryRow = {
  id: string;
  name: string;
  total_score: number;
};

export type Summary = {
  total: number;
  counts: SummaryCounts;
  avgScore: number;
  top: SummaryRow[];
  bottom: SummaryRow[];
};

export type GenerationAvg = {
  generation: string;
  avgScore: number;
  count: number;
};

export type YearAvg = {
  year: number;
  avgScore: number;
  count: number;
};

export type ViewParams = {
  query: string;
  decision: Decision | null;
  generation: string | null;
  yearFrom: number | null;
  yearTo: number | null;
  sortKey: SortKey;
  sortDir: SortDir;
  topN: number;
};

export type ViewModel = {
  generations: string[];
  filteredRows: Row[];
  summary: Summary;
  generationAverages: GenerationAvg[];
  yearlyTrend: YearAvg[];
};

/* Persisted app state (aligned with Rust AppState) */
export type AppState = {
  csvPath: string;
  params: ViewParams;
  selectedId: string;
};

/* CSV validation report types from Rust */
export type CsvIssue = {
  row: number;
  line: number;
  id: string | null;
  message: string;
};

export type CsvValidationReport = {
  path: string;
  totalRows: number;
  okRows: number;
  errorRows: number;
  issues: CsvIssue[];
};