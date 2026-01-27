import type { AnalysisConfig } from "../types";

const STORAGE_KEY = "teaBreed.analysisConfig.v1";

/* Default config matching Rust default_config() */
export const defaultAnalysisConfig: AnalysisConfig = {
  weights: {
    germination: 0.4,
    growth: 0.3,
    disease: 0.2,
    aroma: 0.1,
  },
  keep_threshold: 75,
  review_threshold: 50,
  normalize_to_100: true,
};

/* Load config from localStorage (fallback to default) */
export function loadAnalysisConfig(): AnalysisConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultAnalysisConfig;
    const parsed = JSON.parse(raw) as AnalysisConfig;
    if (!parsed || typeof parsed !== "object") return defaultAnalysisConfig;
    return parsed;
  } catch {
    return defaultAnalysisConfig;
  }
}

/* Save config to localStorage */
export function saveAnalysisConfig(config: AnalysisConfig): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
}

