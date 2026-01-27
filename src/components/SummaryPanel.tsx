import type { ReactElement } from "react";
import {
  Cell,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
} from "recharts";

import type { Decision, Row } from "../types";
import { computeSummary } from "../utils/summary";

/* Format a number for display */
function fmt(n: number): string {
  return n.toFixed(1);
}

/* Map decision to color */
function decisionColor(decision: Decision): string {
  if (decision === "keep") return "#34d399";
  if (decision === "review") return "#fbbf24";
  return "#fb7185";
}

/* Map decision to label */
function decisionLabel(decision: Decision): string {
  return decision;
}

type Props = {
  rows: Row[];
  totalCount: number;
};

/* Summary panel: decision distribution + quick stats */
export function SummaryPanel({ rows, totalCount }: Props): ReactElement {
  const summary = computeSummary(rows, 3);
  const ratio = (n: number): string => {
    if (!summary.total) return "0%";
    return `${Math.round((n / summary.total) * 100)}%`;
  };

  const pieData = (Object.keys(summary.counts) as Decision[])
    .map((k) => ({ decision: k, value: summary.counts[k] }))
    .filter((d) => d.value > 0);

  return (
    <div className="grid grid-cols-12 gap-4">
      <section className="col-span-8">
        <div className="rounded-xl bg-slate-900/60 p-4 shadow-sm ring-1 ring-slate-800">
          <div className="flex items-start justify-between gap-3">
            <div className="flex flex-col gap-1">
              <h2 className="text-sm font-semibold text-slate-200">サマリー</h2>
              <div className="text-xs text-slate-400">
                フィルタ後:{" "}
                <span className="text-slate-200">{summary.total}</span> /{" "}
                {totalCount} 件
              </div>
            </div>

            <div className="text-right">
              <div className="text-xs text-slate-400">平均スコア</div>
              <div className="text-2xl font-semibold">{fmt(summary.avgScore)}</div>
            </div>
          </div>

          <div className="mt-4 grid grid-cols-3 gap-3">
            {(["keep", "review", "discard"] as const).map((d) => (
              <div
                key={d}
                className="rounded-xl bg-slate-950/60 p-3 ring-1 ring-slate-800"
              >
                <div className="text-xs text-slate-400">{decisionLabel(d)}</div>
                <div className="mt-1 flex items-baseline gap-2">
                  <div className="text-2xl font-semibold">
                    {summary.counts[d]}
                  </div>
                  <div className="text-xs text-slate-400">
                    {ratio(summary.counts[d])}
                  </div>
                </div>
              </div>
            ))}
          </div>

          <div className="mt-4 grid grid-cols-2 gap-3">
            <div className="rounded-xl bg-slate-950/60 p-3 ring-1 ring-slate-800">
              <div className="text-xs font-semibold text-slate-300">上位</div>
              <ol className="mt-2 space-y-1 text-sm">
                {summary.top.length ? (
                  summary.top.map((r) => (
                    <li key={r.id} className="flex justify-between gap-2">
                      <span className="truncate text-slate-200">
                        {r.name}
                      </span>
                      <span className="font-mono text-slate-300">
                        {fmt(r.total_score)}
                      </span>
                    </li>
                  ))
                ) : (
                  <li className="text-slate-400">-</li>
                )}
              </ol>
            </div>

            <div className="rounded-xl bg-slate-950/60 p-3 ring-1 ring-slate-800">
              <div className="text-xs font-semibold text-slate-300">下位</div>
              <ol className="mt-2 space-y-1 text-sm">
                {summary.bottom.length ? (
                  summary.bottom.map((r) => (
                    <li key={r.id} className="flex justify-between gap-2">
                      <span className="truncate text-slate-200">
                        {r.name}
                      </span>
                      <span className="font-mono text-slate-300">
                        {fmt(r.total_score)}
                      </span>
                    </li>
                  ))
                ) : (
                  <li className="text-slate-400">-</li>
                )}
              </ol>
            </div>
          </div>
        </div>
      </section>

      <section className="col-span-4">
        <div className="rounded-xl bg-slate-900/60 p-4 shadow-sm ring-1 ring-slate-800">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-slate-200">判定分布</h2>
            <div className="text-xs text-slate-400">{summary.total} 件</div>
          </div>

          <div className="mt-3 h-[240px]">
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Tooltip
                  contentStyle={{
                    background: "#0b1220",
                    border: "1px solid #334155",
                    color: "#e2e8f0",
                  }}
                  formatter={(value: unknown, name: unknown) => {
                    const n = typeof value === "number" ? value : Number(value);
                    const label = Number.isFinite(n) ? n : 0;
                    return [label, String(name)];
                  }}
                />
                <Pie
                  data={pieData}
                  dataKey="value"
                  nameKey="decision"
                  innerRadius={55}
                  outerRadius={85}
                  paddingAngle={2}
                >
                  {pieData.map((d) => (
                    <Cell
                      key={d.decision}
                      fill={decisionColor(d.decision)}
                      stroke="#0b1220"
                      strokeWidth={2}
                    />
                  ))}
                </Pie>
              </PieChart>
            </ResponsiveContainer>
          </div>

          {summary.total === 0 ? (
            <div className="mt-2 text-sm text-slate-400">
              CSV を選択してください
            </div>
          ) : null}
        </div>
      </section>
    </div>
  );
}

