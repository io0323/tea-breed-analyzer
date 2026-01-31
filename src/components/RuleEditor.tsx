import { useState } from "react";
import type { ReactElement } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ValidatorRule } from "../types";

export function RuleEditor({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}): ReactElement | null {
  const [text, setText] = useState<string>("germination_rate <= 80\ngrowth_score == 4");
  const [results, setResults] = useState<Array<ValidatorRule | string>>([]);

  if (!open) return null;

  async function validate() {
    const lines = text
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0);
    const out: Array<ValidatorRule | string> = [];
    for (const l of lines) {
      try {
        const r = await invoke<ValidatorRule>("parse_rule", { rule: l });
        out.push(r);
      } catch (e) {
        out.push(String(e));
      }
    }
    setResults(out);
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-2xl rounded-xl bg-slate-900 p-4 ring-1 ring-slate-800">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-slate-100">ルールエディタ</h3>
          <button type="button" onClick={onClose} className="text-sm text-slate-400">
            閉じる
          </button>
        </div>

        <textarea
          value={text}
          onChange={(e) => setText(e.currentTarget.value)}
          className="mt-3 w-full rounded bg-slate-950 p-2 text-sm font-mono h-32"
        />
        <div className="mt-3 flex gap-2">
          <button onClick={validate} className="rounded bg-sky-600 px-3 py-1 text-sm">
            解析
          </button>
        </div>

        <div className="mt-3 text-sm text-slate-200">
          {results.length === 0 ? (
            <div className="text-slate-400">解析結果がここに表示されます</div>
          ) : (
            <ul className="space-y-1">
              {results.map((r, i) => (
                <li key={i} className="text-xs">
                  {typeof r === "string" ? (
                    <span className="text-rose-300">{r}</span>
                  ) : (
                    <span className="text-sky-300">
                      {r.field} {r.op} {r.value}
                    </span>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}

