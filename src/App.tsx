import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

import { DashboardView } from "./components/DashboardView";
import { GraphView } from "./components/GraphView";
import { SummaryPanel } from "./components/SummaryPanel";
import type {
  AnalysisResult,
  Decision,
  ExportRow,
  Row,
  TeaVariety,
} from "./types";

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

/* Map decision to a button style */
function decisionChipClass(isActive: boolean): string {
  if (isActive) {
    return "rounded-full bg-slate-800 px-3 py-1 text-xs font-semibold " +
      "ring-1 ring-slate-700";
  }
  return "rounded-full bg-slate-950/40 px-3 py-1 text-xs font-medium " +
    "text-slate-300 ring-1 ring-slate-800 hover:bg-slate-900/60";
}

/** TeaBreed Analyzer minimal dashboard */
export default function App() {
  const [csvPath, setCsvPath] = useState<string>("");
  const [rows, setRows] = useState<Row[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");
  const [view, setView] = useState<View>("dashboard");
  const [toast, setToast] = useState<string>("");
  const [isFileDropHover, setIsFileDropHover] = useState<boolean>(false);

  const [query, setQuery] = useState<string>("");
  const [decisionFilter, setDecisionFilter] = useState<Decision | "all">("all");
  const [generationFilter, setGenerationFilter] = useState<string>("all");
  const [yearFrom, setYearFrom] = useState<string>("");
  const [yearTo, setYearTo] = useState<string>("");

  type SortKey = "total_score" | "year" | "name" | "generation" | "decision" | "id";
  type SortDir = "asc" | "desc";

  const [sortKey, setSortKey] = useState<SortKey>("total_score");
  const [sortDir, setSortDir] = useState<SortDir>("desc");

  /* Load a CSV file path and run analysis */
  async function loadAndAnalyze(path: string): Promise<void> {
    setError("");
    setIsLoading(true);

    try {
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

      await loadAndAnalyze(path);
    } finally {
      setIsLoading(false);
    }
  }

  /* Listen to OS file drop events from Tauri */
  useEffect(() => {
    let unlistenDrop: (() => void) | null = null;
    let unlistenHover: (() => void) | null = null;
    let unlistenCancel: (() => void) | null = null;

    const start = async (): Promise<void> => {
      unlistenDrop = await listen<string[]>("tauri://file-drop", async (e) => {
        const paths = e.payload ?? [];
        const first = paths[0];
        setIsFileDropHover(false);

        if (!first) return;
        if (!first.toLowerCase().endsWith(".csv")) {
          setToast("CSV ファイルをドロップしてください");
          return;
        }
        await loadAndAnalyze(first);
      });

      unlistenHover = await listen("tauri://file-drop-hover", () => {
        setIsFileDropHover(true);
      });

      unlistenCancel = await listen("tauri://file-drop-cancelled", () => {
        setIsFileDropHover(false);
      });
    };

    void start();

    return () => {
      if (unlistenDrop) unlistenDrop();
      if (unlistenHover) unlistenHover();
      if (unlistenCancel) unlistenCancel();
    };
  }, []);

  /* Collect unique generation values */
  const generations = useMemo(() => {
    const set = new Set<string>();
    for (const r of rows) set.add(r.generation);
    return Array.from(set.values()).sort((a, b) => a.localeCompare(b));
  }, [rows]);

  /* Apply filters to rows */
  const filteredRows = useMemo(() => {
    const q = query.trim().toLowerCase();
    const from = yearFrom.trim() ? Number(yearFrom) : null;
    const to = yearTo.trim() ? Number(yearTo) : null;
    const hasFrom = from !== null && Number.isFinite(from);
    const hasTo = to !== null && Number.isFinite(to);

    return rows.filter((r) => {
      if (decisionFilter !== "all" && r.decision !== decisionFilter) {
        return false;
      }
      if (generationFilter !== "all" && r.generation !== generationFilter) {
        return false;
      }
      if (hasFrom && r.year < (from as number)) {
        return false;
      }
      if (hasTo && r.year > (to as number)) {
        return false;
      }
      if (q) {
        const hay = `${r.id} ${r.name} ${r.location} ${r.generation}`
          .toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    });
  }, [rows, query, decisionFilter, generationFilter, yearFrom, yearTo]);

  /* Sort rows for table/export (independent from chart order) */
  const displayRows = useMemo(() => {
    const decisionOrder: Record<Decision, number> = {
      keep: 0,
      review: 1,
      discard: 2,
    };

    const sorted = [...filteredRows].sort((a, b) => {
      let cmp = 0;

      if (sortKey === "total_score") cmp = a.total_score - b.total_score;
      else if (sortKey === "year") cmp = a.year - b.year;
      else if (sortKey === "name") cmp = a.name.localeCompare(b.name);
      else if (sortKey === "generation") cmp = a.generation.localeCompare(b.generation);
      else if (sortKey === "decision") cmp = decisionOrder[a.decision] - decisionOrder[b.decision];
      else if (sortKey === "id") cmp = a.id.localeCompare(b.id);

      if (cmp === 0) {
        /* Stable tie-breaker */
        cmp = a.id.localeCompare(b.id);
      }

      return sortDir === "asc" ? cmp : -cmp;
    });

    return sorted;
  }, [filteredRows, sortKey, sortDir]);

  /* Ensure selection exists in filtered list */
  const effectiveSelectedId = useMemo(() => {
    if (displayRows.some((r) => r.id === selectedId)) {
      return selectedId;
    }
    return displayRows[0]?.id ?? "";
  }, [displayRows, selectedId]);

  /* Save currently filtered rows to a CSV file */
  async function exportFilteredCsv(): Promise<void> {
    setError("");
    setToast("");

    if (displayRows.length === 0) {
      setToast("エクスポート対象がありません（フィルタ後 0 件）");
      return;
    }

    try {
      const path = await save({
        filters: [{ name: "CSV", extensions: ["csv"] }],
        defaultPath: "tea_breed_analysis.csv",
      });

      if (!path) return;

      const exportRows: ExportRow[] = displayRows;
      await invoke("save_analysis_csv", { path, rows: exportRows });
      setToast("CSV を保存しました");
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
    }
  }

  return (
    <div className="min-h-full bg-slate-950 text-slate-100">
      {isFileDropHover ? (
        <div className="fixed inset-0 z-50 bg-slate-950/70 backdrop-blur-sm">
          <div className="mx-auto flex h-full max-w-3xl items-center justify-center p-6">
            <div className="w-full rounded-2xl border border-dashed border-slate-500/60 bg-slate-900/40 p-10 text-center shadow-sm">
              <div className="text-lg font-semibold text-slate-100">
                CSV をここにドロップ
              </div>
              <div className="mt-2 text-sm text-slate-400">
                ドロップすると自動で解析します
              </div>
            </div>
          </div>
        </div>
      ) : null}

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

            <button
              type="button"
              onClick={exportFilteredCsv}
              className="rounded-lg bg-slate-900/60 px-4 py-2 text-sm font-medium
                text-slate-200 shadow-sm ring-1 ring-slate-800
                hover:bg-slate-900 disabled:cursor-not-allowed disabled:opacity-60"
              disabled={isLoading || rows.length === 0}
            >
              CSV 出力
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

        {toast ? (
          <div
            className="rounded-lg bg-sky-500/10 p-3 text-sm text-sky-200
              ring-1 ring-sky-500/20"
          >
            {toast}
          </div>
        ) : null}

        {csvPath ? (
          <div className="text-xs text-slate-400">
            読み込み元: <span className="text-slate-200">{csvPath}</span>
          </div>
        ) : null}

        <div className="rounded-xl bg-slate-900/40 p-3 ring-1 ring-slate-800">
          <div className="grid grid-cols-12 gap-3">
            <div className="col-span-4">
              <div className="text-xs font-medium text-slate-300">検索</div>
              <input
                value={query}
                onChange={(e) => setQuery(e.currentTarget.value)}
                placeholder="ID / 名前 / 産地 / 世代"
                className="mt-1 w-full rounded-lg bg-slate-950/40 px-3 py-2
                  text-sm text-slate-100 ring-1 ring-slate-800
                  placeholder:text-slate-500"
              />
            </div>

            <div className="col-span-4">
              <div className="text-xs font-medium text-slate-300">判定</div>
              <div className="mt-1 flex flex-wrap gap-2">
                {(["all", "keep", "review", "discard"] as const).map((d) => (
                  <button
                    key={d}
                    type="button"
                    onClick={() => setDecisionFilter(d)}
                    className={decisionChipClass(decisionFilter === d)}
                  >
                    {d}
                  </button>
                ))}
              </div>
            </div>

            <div className="col-span-4">
              <div className="text-xs font-medium text-slate-300">世代</div>
              <select
                value={generationFilter}
                onChange={(e) => setGenerationFilter(e.currentTarget.value)}
                className="mt-1 w-full rounded-lg bg-slate-950/40 px-3 py-2
                  text-sm text-slate-100 ring-1 ring-slate-800"
              >
                <option value="all">all</option>
                {generations.map((g) => (
                  <option key={g} value={g}>
                    {g}
                  </option>
                ))}
              </select>
            </div>

            <div className="col-span-2">
              <div className="text-xs font-medium text-slate-300">年 From</div>
              <input
                value={yearFrom}
                onChange={(e) => setYearFrom(e.currentTarget.value)}
                placeholder="例: 2023"
                className="mt-1 w-full rounded-lg bg-slate-950/40 px-3 py-2
                  text-sm text-slate-100 ring-1 ring-slate-800
                  placeholder:text-slate-500"
              />
            </div>
            <div className="col-span-2">
              <div className="text-xs font-medium text-slate-300">年 To</div>
              <input
                value={yearTo}
                onChange={(e) => setYearTo(e.currentTarget.value)}
                placeholder="例: 2025"
                className="mt-1 w-full rounded-lg bg-slate-950/40 px-3 py-2
                  text-sm text-slate-100 ring-1 ring-slate-800
                  placeholder:text-slate-500"
              />
            </div>

            <div className="col-span-8 flex items-end justify-end">
              <div className="flex items-center gap-3">
                <div className="text-xs text-slate-400">
                  フィルタ後:{" "}
                  <span className="text-slate-200">{filteredRows.length}</span>{" "}
                  / {rows.length} 件
                </div>
                <button
                  type="button"
                  onClick={() => {
                    setQuery("");
                    setDecisionFilter("all");
                    setGenerationFilter("all");
                    setYearFrom("");
                    setYearTo("");
                  }}
                  className="rounded-lg bg-slate-950/40 px-3 py-2 text-xs
                    text-slate-200 ring-1 ring-slate-800 hover:bg-slate-900/60"
                >
                  フィルタをリセット
                </button>
              </div>
            </div>
          </div>
        </div>

        <SummaryPanel
          rows={filteredRows}
          totalCount={rows.length}
          activeDecision={decisionFilter}
          onDecisionClick={(d) => {
            setView("dashboard");
            setDecisionFilter(decisionFilter === d ? "all" : d);
          }}
        />

        {view === "dashboard" ? (
          <DashboardView
            rows={displayRows}
            totalCount={rows.length}
            selectedId={effectiveSelectedId}
            onSelectId={setSelectedId}
            sortKey={sortKey}
            sortDir={sortDir}
            onSortChange={(nextKey) => {
              if (nextKey === sortKey) {
                setSortDir(sortDir === "asc" ? "desc" : "asc");
              } else {
                setSortKey(nextKey);
                setSortDir(nextKey === "total_score" ? "desc" : "asc");
              }
            }}
          />
        ) : (
          <GraphView rows={filteredRows} />
        )}
      </div>
    </div>
  );
}
