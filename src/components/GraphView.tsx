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
import type { GenerationAvg, YearAvg } from "../types";

type Props = {
  generationAverages: GenerationAvg[];
  yearlyTrend: YearAvg[];
  total: number;
};

/* Graph view: generation average (bar) + yearly trend (line) */
export function GraphView({
  generationAverages,
  yearlyTrend,
  total,
}: Props): ReactElement {
  const generationData = generationAverages.map((d) => ({
    generation: d.generation,
    avg_score: d.avgScore,
    count: d.count,
  }));
  const yearlyData = yearlyTrend.map((d) => ({
    year: d.year,
    avg_score: d.avgScore,
    count: d.count,
  }));

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

          {total === 0 ? (
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

          {total === 0 ? (
            <div className="mt-3 text-sm text-slate-400">
              CSV を選択してください
            </div>
          ) : null}
        </div>
      </section>
    </div>
  );
}

