import type { Decision, Row } from "../types";

/* Summary aggregates for quick decision-making */
export type DecisionCounts = Record<Decision, number>;

export type Summary = {
  total: number;
  counts: DecisionCounts;
  avgScore: number;
  top: Row[];
  bottom: Row[];
};

/* Create a stable counts object */
export function emptyCounts(): DecisionCounts {
  return { keep: 0, review: 0, discard: 0 };
}

/* Compute summary from rows (already filtered) */
export function computeSummary(rows: Row[], topN = 3): Summary {
  const counts = emptyCounts();
  let sum = 0;

  for (const r of rows) {
    counts[r.decision] += 1;
    sum += r.total_score;
  }

  const sorted = [...rows].sort((a, b) => b.total_score - a.total_score);
  const top = sorted.slice(0, topN);
  const bottom = sorted.slice(Math.max(0, sorted.length - topN)).reverse();

  return {
    total: rows.length,
    counts,
    avgScore: rows.length ? sum / rows.length : 0,
    top,
    bottom,
  };
}

