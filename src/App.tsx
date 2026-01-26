import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

/* Tea variety model aligned with Rust */
type TeaVariety = {
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
type AnalysisResult = {
  id: string;
  total_score: number;
  decision: "keep" | "review" | "discard";
};

type Row = TeaVariety & AnalysisResult;

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

/** TeaBreed Analyzer minimal dashboard */
export default function App() {
  const [csvPath, setCsvPath] = useState<string>("");
  const [rows, setRows] = useState<Row[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");

  const selected = useMemo(() => {
    return rows.find((r) => r.id === selectedId) ?? null;
  }, [rows, selectedId]);

  /* Pick a CSV file and run analysis automatically */
  async function pickCsvAndAnalyze(): Promise<void> {
    setError("");
    setIsLoading(true);

    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });

      if (!path || Array.isArray(path)) {
        return;
      }

      setCsvPath(path);

      const data = await invoke<TeaVariety[]>("load_csv", { path });
      const results = await invoke<AnalysisResult[]>("analyze_varieties", {
        data,
      });

      const resultById = new Map(results.map((r) => [r.id, r]));
      const merged = data
        .map((v) => {
          const a = resultById.get(v.id);
          if (!a) return null;
          return { ...v, ...a };
        })
        .filter((v): v is Row => v !== null)
        .sort((a, b) => b.total_score - a.total_score);

      setRows(merged);
      setSelectedId(merged[0]?.id ?? "");
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="min-h-full bg-slate-950 text-slate-100">
      <div className="mx-auto flex max-w-6xl flex-col gap-4 p-4">
        <header className="flex items-center justify-between">
          <div className="flex flex-col gap-1">
            <h1 className="text-xl font-semibold tracking-tight">
              TeaBreed Analyzer
            </h1>
            <p className="text-sm text-slate-400">
              CSV 読み込み → 自動解析（keep / review / discard）
            </p>
          </div>

          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={pickCsvAndAnalyze}
              className="rounded-lg bg-slate-800 px-4 py-2 text-sm font-medium
                shadow-sm ring-1 ring-slate-700 hover:bg-slate-750
                disabled:cursor-not-allowed disabled:opacity-60"
              disabled={isLoading}
            >
              {isLoading ? "読み込み中..." : "CSV を選択"}
            </button>
          </div>
        </header>

        {error ? (
          <div
            className="rounded-lg bg-rose-500/10 p-3 text-sm text-rose-200
              ring-1 ring-rose-500/20"
          >
            {error}
          </div>
        ) : null}

        {csvPath ? (
          <div className="text-xs text-slate-400">
            読み込み元: <span className="text-slate-200">{csvPath}</span>
          </div>
        ) : null}

        <div className="grid grid-cols-12 gap-4">
          <section className="col-span-7">
            <div className="rounded-xl bg-slate-900/60 p-3 shadow-sm ring-1 ring-slate-800">
              <div className="mb-2 flex items-center justify-between">
                <h2 className="text-sm font-semibold text-slate-200">
                  品種一覧
                </h2>
                <div className="text-xs text-slate-400">
                  {rows.length} 件
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
                          onClick={() => setSelectedId(r.id)}
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
              <h2 className="text-sm font-semibold text-slate-200">
                選択中品種
              </h2>

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
      </div>
    </div>
  );
}
