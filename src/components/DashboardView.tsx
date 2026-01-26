import type { Row } from "../types";
import type { ReactElement } from "react";

/* Convert decision into badge styles */
function decisionStyles(decision: Row["decision"]): {
  badge: string;
  label: string;
} {
  if (decision === "keep") {
    return {
      badge: "bg-emerald-500/20 text-emerald-200 ring-1 ring-emerald-500/30",
      label: "keep",
    };
  }
  if (decision === "review") {
    return {
      badge: "bg-amber-500/20 text-amber-200 ring-1 ring-amber-500/30",
      label: "review",
    };
  }
  return {
    badge: "bg-rose-500/20 text-rose-200 ring-1 ring-rose-500/30",
    label: "discard",
  };
}

/* Format score for display */
function formatScore(score: number): string {
  return score.toFixed(1);
}

type Props = {
  rows: Row[];
  totalCount?: number;
  selectedId: string;
  onSelectId: (id: string) => void;
};

/* Minimal dashboard: table + selected card */
export function DashboardView({
  rows,
  totalCount,
  selectedId,
  onSelectId,
}: Props): ReactElement {
  const selected = rows.find((r) => r.id === selectedId) ?? null;

  return (
    <div className="grid grid-cols-12 gap-4">
      <section className="col-span-7">
        <div className="rounded-xl bg-slate-900/60 p-3 shadow-sm ring-1 ring-slate-800">
          <div className="mb-2 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-slate-200">品種一覧</h2>
            <div className="text-xs text-slate-400">
              {rows.length} 件
              {typeof totalCount === "number" ? (
                <>
                  {" "}
                  / <span className="text-slate-300">{totalCount}</span> 件
                </>
              ) : null}
            </div>
          </div>

          <div className="max-h-[520px] overflow-auto rounded-lg ring-1 ring-slate-800">
            <table className="w-full border-collapse text-left text-sm">
              <thead className="sticky top-0 bg-slate-950/80 backdrop-blur">
                <tr className="text-xs text-slate-300">
                  <th className="px-3 py-2 font-medium">ID</th>
                  <th className="px-3 py-2 font-medium">名前</th>
                  <th className="px-3 py-2 font-medium">世代</th>
                  <th className="px-3 py-2 font-medium">年</th>
                  <th className="px-3 py-2 font-medium">総合</th>
                  <th className="px-3 py-2 font-medium">判定</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => {
                  const styles = decisionStyles(r.decision);
                  const isSelected = r.id === selectedId;

                  return (
                    <tr
                      key={r.id}
                      onClick={() => onSelectId(r.id)}
                      className={
                        "cursor-pointer border-t border-slate-800/70 " +
                        (isSelected
                          ? "bg-slate-800/40"
                          : "hover:bg-slate-800/20")
                      }
                    >
                      <td className="px-3 py-2 font-mono text-xs text-slate-300">
                        {r.id}
                      </td>
                      <td className="px-3 py-2">{r.name}</td>
                      <td className="px-3 py-2 text-slate-300">
                        {r.generation}
                      </td>
                      <td className="px-3 py-2 text-slate-300">{r.year}</td>
                      <td className="px-3 py-2 font-semibold">
                        {formatScore(r.total_score)}
                      </td>
                      <td className="px-3 py-2">
                        <span
                          className={
                            "inline-flex items-center rounded-full px-2 py-1 " +
                            "text-xs font-semibold " +
                            styles.badge
                          }
                        >
                          {styles.label}
                        </span>
                      </td>
                    </tr>
                  );
                })}
                {rows.length === 0 ? (
                  <tr>
                    <td
                      colSpan={6}
                      className="px-3 py-8 text-center text-sm text-slate-400"
                    >
                      CSV を選択してください
                    </td>
                  </tr>
                ) : null}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <section className="col-span-5">
        <div className="rounded-xl bg-slate-900/60 p-4 shadow-sm ring-1 ring-slate-800">
          <h2 className="text-sm font-semibold text-slate-200">選択中品種</h2>

          {selected ? (
            <div className="mt-3 flex flex-col gap-4">
              <div className="rounded-xl bg-slate-950/60 p-4 ring-1 ring-slate-800">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="truncate text-lg font-semibold">
                      {selected.name}
                    </div>
                    <div className="mt-1 text-xs text-slate-400">
                      {selected.id} ・ {selected.generation} ・{" "}
                      {selected.location} ・ {selected.year}
                    </div>
                  </div>
                  <span
                    className={
                      "inline-flex items-center rounded-full px-2 py-1 " +
                      "text-xs font-semibold " +
                      decisionStyles(selected.decision).badge
                    }
                  >
                    {selected.decision}
                  </span>
                </div>

                <div className="mt-4">
                  <div className="text-xs text-slate-400">総合スコア</div>
                  <div className="mt-1 text-4xl font-bold tracking-tight">
                    {formatScore(selected.total_score)}
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="rounded-xl bg-slate-950/60 p-3 ring-1 ring-slate-800">
                  <div className="text-xs text-slate-400">発芽率</div>
                  <div className="mt-1 text-2xl font-semibold">
                    {formatScore(selected.germination_rate)}%
                  </div>
                </div>
                <div className="rounded-xl bg-slate-950/60 p-3 ring-1 ring-slate-800">
                  <div className="text-xs text-slate-400">生育</div>
                  <div className="mt-1 text-2xl font-semibold">
                    {selected.growth_score}/5
                  </div>
                </div>
                <div className="rounded-xl bg-slate-950/60 p-3 ring-1 ring-slate-800">
                  <div className="text-xs text-slate-400">耐病性</div>
                  <div className="mt-1 text-2xl font-semibold">
                    {selected.disease_resistance}/5
                  </div>
                </div>
                <div className="rounded-xl bg-slate-950/60 p-3 ring-1 ring-slate-800">
                  <div className="text-xs text-slate-400">香り</div>
                  <div className="mt-1 text-2xl font-semibold">
                    {selected.aroma_score}/5
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="mt-3 text-sm text-slate-400">
              品種を選択してください
            </div>
          )}
        </div>
      </section>
    </div>
  );
}

