import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import { DashboardView } from "./components/DashboardView";
import { GraphView } from "./components/GraphView";
import type { AnalysisResult, Row, TeaVariety } from "./types";

type View = "dashboard" | "graphs";

/* Nav button styling helper */
function navButtonClass(isActive: boolean): string {
  if (isActive) {
    return "rounded-lg bg-slate-800 px-3 py-2 text-sm font-semibold " +
      "ring-1 ring-slate-700";
  }
  return "rounded-lg px-3 py-2 text-sm font-medium text-slate-300 " +
    "hover:bg-slate-900/60 hover:text-slate-100";
}

/** TeaBreed Analyzer minimal dashboard */
export default function App() {
  const [csvPath, setCsvPath] = useState<string>("");
  const [rows, setRows] = useState<Row[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");
  const [view, setView] = useState<View>("dashboard");

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
            <nav className="flex items-center gap-1 rounded-xl bg-slate-950/40 p-1 ring-1 ring-slate-800">
              <button
                type="button"
                onClick={() => setView("dashboard")}
                className={navButtonClass(view === "dashboard")}
              >
                ダッシュボード
              </button>
              <button
                type="button"
                onClick={() => setView("graphs")}
                className={navButtonClass(view === "graphs")}
              >
                グラフ
              </button>
            </nav>

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

        {view === "dashboard" ? (
          <DashboardView
            rows={rows}
            selectedId={selectedId}
            onSelectId={setSelectedId}
          />
        ) : (
          <GraphView rows={rows} />
        )}
      </div>
    </div>
  );
}
