import {
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import type { ReactElement } from "react";
import type { Row } from "../types";

type GenerationAvg = {
  generation: string;
  avg_score: number;
  count: number;
};

type YearAvg = {
  year: number;
  avg_score: number;
  count: number;
};

/* Compute generation average scores */
function computeGenerationAverages(rows: Row[]): GenerationAvg[] {
  const acc = new Map<string, { sum: number; count: number }>();

  for (const r of rows) {
    const cur = acc.get(r.generation) ?? { sum: 0, count: 0 };
    cur.sum += r.total_score;
    cur.count += 1;
    acc.set(r.generation, cur);
  }

  return Array.from(acc.entries())
    .map(([generation, v]) => ({
      generation,
      avg_score: v.count ? v.sum / v.count : 0,
      count: v.count,
    }))
    .sort((a, b) => a.generation.localeCompare(b.generation));
}

/* Compute yearly trend (average total_score by year) */
function computeYearlyTrend(rows: Row[]): YearAvg[] {
  const acc = new Map<number, { sum: number; count: number }>();

  for (const r of rows) {
    const cur = acc.get(r.year) ?? { sum: 0, count: 0 };
    cur.sum += r.total_score;
    cur.count += 1;
    acc.set(r.year, cur);
  }

  return Array.from(acc.entries())
    .map(([year, v]) => ({
      year,
      avg_score: v.count ? v.sum / v.count : 0,
      count: v.count,
    }))
    .sort((a, b) => a.year - b.year);
}

type Props = {
  rows: Row[];
};

/* Graph view: generation average (bar) + yearly trend (line) */
export function GraphView({ rows }: Props): ReactElement {
  const generationData = computeGenerationAverages(rows);
  const yearlyData = computeYearlyTrend(rows);

  return (
    <div className="grid grid-cols-12 gap-4">
      <section className="col-span-6">
        <div className="rounded-xl bg-slate-900/60 p-4 shadow-sm ring-1 ring-slate-800">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-slate-200">
              世代別スコア平均
            </h2>
            <div className="text-xs text-slate-400">
              {generationData.reduce((n, d) => n + d.count, 0)} 件
            </div>
          </div>

          <div className="h-[360px]">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={generationData}>
                <CartesianGrid stroke="#1f2937" strokeDasharray="3 3" />
                <XAxis
                  dataKey="generation"
                  tick={{ fill: "#cbd5e1", fontSize: 12 }}
                  axisLine={{ stroke: "#334155" }}
                  tickLine={{ stroke: "#334155" }}
                />
                <YAxis
                  tick={{ fill: "#cbd5e1", fontSize: 12 }}
                  axisLine={{ stroke: "#334155" }}
                  tickLine={{ stroke: "#334155" }}
                  domain={[0, 100]}
                />
                <Tooltip
                  contentStyle={{
                    background: "#0b1220",
                    border: "1px solid #334155",
                    color: "#e2e8f0",
                  }}
                  labelStyle={{ color: "#e2e8f0" }}
                  formatter={(value: unknown) => {
                    const n = typeof value === "number" ? value : Number(value);
                    const label = Number.isFinite(n) ? n.toFixed(1) : "-";
                    return [label, "平均"];
                  }}
                />
                <Bar dataKey="avg_score" fill="#38bdf8" radius={[6, 6, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>

          {rows.length === 0 ? (
            <div className="mt-3 text-sm text-slate-400">
              CSV を選択してください
            </div>
          ) : null}
        </div>
      </section>

      <section className="col-span-6">
        <div className="rounded-xl bg-slate-900/60 p-4 shadow-sm ring-1 ring-slate-800">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-slate-200">年次推移</h2>
            <div className="text-xs text-slate-400">
              {yearlyData.reduce((n, d) => n + d.count, 0)} 件
            </div>
          </div>

          <div className="h-[360px]">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={yearlyData}>
                <CartesianGrid stroke="#1f2937" strokeDasharray="3 3" />
                <XAxis
                  dataKey="year"
                  tick={{ fill: "#cbd5e1", fontSize: 12 }}
                  axisLine={{ stroke: "#334155" }}
                  tickLine={{ stroke: "#334155" }}
                />
                <YAxis
                  tick={{ fill: "#cbd5e1", fontSize: 12 }}
                  axisLine={{ stroke: "#334155" }}
                  tickLine={{ stroke: "#334155" }}
                  domain={[0, 100]}
                />
                <Tooltip
                  contentStyle={{
                    background: "#0b1220",
                    border: "1px solid #334155",
                    color: "#e2e8f0",
                  }}
                  labelStyle={{ color: "#e2e8f0" }}
                  formatter={(value: unknown) => {
                    const n = typeof value === "number" ? value : Number(value);
                    const label = Number.isFinite(n) ? n.toFixed(1) : "-";
                    return [label, "平均"];
                  }}
                />
                <Line
                  type="monotone"
                  dataKey="avg_score"
                  stroke="#a78bfa"
                  strokeWidth={2}
                  dot={{ r: 3 }}
                  activeDot={{ r: 5 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>

          {rows.length === 0 ? (
            <div className="mt-3 text-sm text-slate-400">
              CSV を選択してください
            </div>
          ) : null}
        </div>
      </section>
    </div>
  );
}

