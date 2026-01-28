import type { ReactElement } from "react";
import type { CsvValidationReport } from "../types";

type Props = {
  open: boolean;
  onClose: () => void;
  report: CsvValidationReport | null;
  onSelectRow?: (id: string) => void;
};

export function ValidationModal({ open, onClose, report }: Props): ReactElement | null {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-3xl rounded-xl bg-slate-900 p-4 ring-1 ring-slate-800">
        <div className="flex items-start justify-between">
          <h3 className="text-lg font-semibold text-slate-100">CSV 検証レポート</h3>
          <button
            type="button"
            onClick={onClose}
            className="text-sm text-slate-400 hover:text-slate-200"
          >
            閉じる
          </button>
        </div>

        <div className="mt-3 text-sm text-slate-200">
          {report ? (
            <>
              <div className="mb-2 text-xs text-slate-400">path: {report.path}</div>
              <div className="mb-3 grid grid-cols-3 gap-2">
                <div>総行: {report.totalRows}</div>
                <div className="text-emerald-300">正常: {report.okRows}</div>
                <div className="text-rose-300">異常: {report.errorRows}</div>
              </div>

              <div className="max-h-64 overflow-auto rounded-md border border-slate-800 bg-slate-950/30 p-2 text-xs">
                {report.issues.length === 0 ? (
                  <div className="text-slate-400">問題は見つかりませんでした。</div>
                ) : (
                  <ol className="space-y-2">
                    {report.issues.map((it) => (
                      <li key={`${it.row}-${it.line}`} className="text-sm">
                        <div className="flex items-center justify-between gap-2">
                          <div className="text-slate-300 font-mono">
                            row:{it.row} line:{it.line} id:{it.id ?? "-"}
                          </div>
                          {it.id ? (
                            <button
                              type="button"
                              className="text-xs text-sky-300 hover:underline"
                              onClick={() => {
                                // call parent via DOM event - replaced by prop in App usage
                                const evt = new CustomEvent("validation:select", {
                                  detail: { id: it.id },
                                });
                                window.dispatchEvent(evt);
                                onClose();
                              }}
                            >
                              テーブルで表示
                            </button>
                          ) : null}
                        </div>
                        <div className="text-rose-300 mt-1">{it.message}</div>
                      </li>
                    ))}
                  </ol>
                )}
              </div>
            </>
          ) : (
            <div className="text-slate-400">レポートがありません</div>
          )}
        </div>
      </div>
    </div>
  );
}

