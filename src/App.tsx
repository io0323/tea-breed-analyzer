import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

import { DashboardView } from "./components/DashboardView";
import { GraphView } from "./components/GraphView";
import { SummaryPanel } from "./components/SummaryPanel";
import type {
  AppState,
  Decision,
  ExportRow,
  Row,
  SortDir,
  SortKey,
  ViewModel,
  ViewParams,
} from "./types";
import { ValidationModal } from "./components/ValidationModal";
import type { CsvValidationReport } from "./types";

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
  const [isValidationOpen, setIsValidationOpen] = useState<boolean>(false);
  const [validationReport, setValidationReport] = useState<CsvValidationReport | null>(null);

  const [query, setQuery] = useState<string>("");
  const [decisionFilter, setDecisionFilter] = useState<Decision | "all">("all");
  const [generationFilter, setGenerationFilter] = useState<string>("all");
  const [yearFrom, setYearFrom] = useState<string>("");
  const [yearTo, setYearTo] = useState<string>("");

  const [sortKey, setSortKey] = useState<SortKey>("total_score");
  const [sortDir, setSortDir] = useState<SortDir>("desc");

  const hasRestoredStateRef = useRef<boolean>(false);

  const [viewModel, setViewModel] = useState<ViewModel>({
    generations: [],
    filteredRows: [],
    summary: {
      total: 0,
      counts: { keep: 0, review: 0, discard: 0 },
      avgScore: 0,
      top: [],
      bottom: [],
    },
    generationAverages: [],
    yearlyTrend: [],
  });

  /* Load a CSV file path and run analysis */
  async function loadAndAnalyze(
    path: string,
    preferredSelectedId?: string,
  ): Promise<void> {
    setError("");
    setIsLoading(true);

    try {
      setCsvPath(path);
      const merged = await invoke<Row[]>("load_and_analyze_csv", { path });
      setRows(merged);
      if (preferredSelectedId && merged.some((r) => r.id === preferredSelectedId)) {
        setSelectedId(preferredSelectedId);
      } else {
        setSelectedId(merged[0]?.id ?? "");
      }
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

  /* Restore last app state on startup */
  useEffect(() => {
    let cancelled = false;

    const restore = async (): Promise<void> => {
      try {
        const state = await invoke<AppState | null>("load_app_state");
        if (cancelled) return;
        if (!state) {
          hasRestoredStateRef.current = true;
          return;
        }

        setQuery(state.params.query);
        setDecisionFilter(state.params.decision ?? "all");
        setGenerationFilter(state.params.generation ?? "all");
        setYearFrom(state.params.yearFrom !== null ? String(state.params.yearFrom) : "");
        setYearTo(state.params.yearTo !== null ? String(state.params.yearTo) : "");
        setSortKey(state.params.sortKey);
        setSortDir(state.params.sortDir);

        if (state.csvPath) {
          await loadAndAnalyze(state.csvPath, state.selectedId);
        } else {
          setCsvPath("");
          setRows([]);
          setSelectedId(state.selectedId ?? "");
        }
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        setError(message);
      } finally {
        hasRestoredStateRef.current = true;
      }
    };

    void restore();
    return () => {
      cancelled = true;
    };
  }, []);

  /* Compute filters/sort/summary/charts in Rust (view model) */
  useEffect(() => {
    let cancelled = false;

    const run = async (): Promise<void> => {
      if (rows.length === 0) {
        setViewModel((cur) => ({
          ...cur,
          generations: [],
          filteredRows: [],
          summary: {
            total: 0,
            counts: { keep: 0, review: 0, discard: 0 },
            avgScore: 0,
            top: [],
            bottom: [],
          },
          generationAverages: [],
          yearlyTrend: [],
        }));
        return;
      }

      const fromRaw = yearFrom.trim() ? Number(yearFrom) : null;
      const toRaw = yearTo.trim() ? Number(yearTo) : null;
      const yearFromNum = fromRaw !== null && Number.isFinite(fromRaw)
        ? Math.trunc(fromRaw)
        : null;
      const yearToNum = toRaw !== null && Number.isFinite(toRaw)
        ? Math.trunc(toRaw)
        : null;

      const params: ViewParams = {
        query,
        decision: decisionFilter === "all" ? null : decisionFilter,
        generation: generationFilter === "all" ? null : generationFilter,
        yearFrom: yearFromNum,
        yearTo: yearToNum,
        sortKey,
        sortDir,
        topN: 3,
      };

      try {
        const next = await invoke<ViewModel>("compute_view_model", { rows, params });
        if (cancelled) return;
        setViewModel(next);
      } catch (e) {
        if (cancelled) return;
        const message = e instanceof Error ? e.message : String(e);
        setError(message);
      }
    };

    void run();
    return () => {
      cancelled = true;
    };
  }, [
    rows,
    query,
    decisionFilter,
    generationFilter,
    yearFrom,
    yearTo,
    sortKey,
    sortDir,
  ]);

  const generations = viewModel.generations;
  const displayRows = viewModel.filteredRows;
  const filteredCount = viewModel.summary.total;

  /* Ensure selection exists in filtered list */
  const effectiveSelectedId = useMemo(() => {
    if (displayRows.some((r) => r.id === selectedId)) {
      return selectedId;
    }
    return displayRows[0]?.id ?? "";
  }, [displayRows, selectedId]);

  /* Persist app state (debounced) */
  useEffect(() => {
    if (!hasRestoredStateRef.current) return;

    const params: ViewParams = {
      query,
      decision: decisionFilter === "all" ? null : decisionFilter,
      generation: generationFilter === "all" ? null : generationFilter,
      yearFrom: yearFrom.trim() ? Number(yearFrom) : null,
      yearTo: yearTo.trim() ? Number(yearTo) : null,
      sortKey,
      sortDir,
      topN: 3,
    };

    const state: AppState = {
      csvPath,
      params,
      selectedId: effectiveSelectedId,
    };

    const handle = window.setTimeout(() => {
      void invoke("save_app_state", { state });
    }, 500);

    return () => {
      window.clearTimeout(handle);
    };
  }, [
    csvPath,
    query,
    decisionFilter,
    generationFilter,
    yearFrom,
    yearTo,
    sortKey,
    sortDir,
    effectiveSelectedId,
  ]);

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

  /* Save filtered rows as JSON */
  async function exportFilteredJson(): Promise<void> {
    setError("");
    setToast("");

    if (displayRows.length === 0) {
      setToast("エクスポート対象がありません（フィルタ後 0 件）");
      return;
    }

    try {
      const path = await save({
        filters: [{ name: "JSON", extensions: ["json"] }],
        defaultPath: "tea_breed_analysis.json",
      });

      if (!path) return;

      const exportRows: ExportRow[] = displayRows;
      await invoke("save_analysis_json", { path, rows: exportRows });
      setToast("JSON を保存しました");
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
    }
  }

  /* Save a Markdown report for the current filtered view */
  async function exportReportMarkdown(): Promise<void> {
    setError("");
    setToast("");

    try {
      const path = await save({
        filters: [{ name: "Markdown", extensions: ["md"] }],
        defaultPath: "tea_breed_report.md",
      });

      if (!path) return;
      await invoke("save_report_markdown", { path, viewModel });
      setToast("Markdown レポートを保存しました");
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
    }
  }

  return (
    <>
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
            onClick={async () => {
                setError("");
                setIsLoading(true);
                try {
                  if (!csvPath) {
                    setToast("先に CSV を選択するか、ファイルパスを入力してください");
                    return;
                  }
                  const rep = await invoke<CsvValidationReport>("validate_csv", { path: csvPath });
                  setValidationReport(rep);
                  setIsValidationOpen(true);
                } catch (e) {
                  const message = e instanceof Error ? e.message : String(e);
                  setError(message);
                } finally {
                  setIsLoading(false);
                }
              }}
              className="rounded-lg bg-slate-800 px-4 py-2 text-sm font-medium
                shadow-sm ring-1 ring-slate-700 hover:bg-slate-750
                disabled:cursor-not-allowed disabled:opacity-60"
              disabled={isLoading || !csvPath}
            >
              CSV 検証
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

            <button
              type="button"
              onClick={exportFilteredJson}
              className="rounded-lg bg-slate-900/60 px-4 py-2 text-sm font-medium
                text-slate-200 shadow-sm ring-1 ring-slate-800
                hover:bg-slate-900 disabled:cursor-not-allowed disabled:opacity-60"
              disabled={isLoading || rows.length === 0}
            >
              JSON 出力
            </button>

            <button
              type="button"
              onClick={exportReportMarkdown}
              className="rounded-lg bg-slate-900/60 px-4 py-2 text-sm font-medium
                text-slate-200 shadow-sm ring-1 ring-slate-800
                hover:bg-slate-900 disabled:cursor-not-allowed disabled:opacity-60"
              disabled={isLoading}
            >
              レポート(.md)
            </button>
          </div>
        </header>

        {error ? (
          <div
            className="rounded-lg bg-rose-500/10 p-3 text-sm text-rose-200
              ring-1 ring-rose-500/20 whitespace-pre-wrap break-words"
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
                  <span className="text-slate-200">{filteredCount}</span>{" "}
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
          summary={viewModel.summary}
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
          <GraphView
            generationAverages={viewModel.generationAverages}
            yearlyTrend={viewModel.yearlyTrend}
            total={viewModel.summary.total}
          />
        )}
      </div>
    </div>
    <ValidationModal
      open={isValidationOpen}
      onClose={() => setIsValidationOpen(false)}
      report={validationReport}
      onSelectRow={(id) => {
        setSelectedId(id);
        setView("dashboard");
      }}
    />
    </>
  );
}
