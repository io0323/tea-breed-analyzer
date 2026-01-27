import type { ReactElement } from "react";

import type { AnalysisConfig } from "../types";
import { defaultAnalysisConfig } from "../utils/analysisConfig";

type Props = {
  open: boolean;
  config: AnalysisConfig;
  onClose: () => void;
  onChange: (next: AnalysisConfig) => void;
  onApply: () => void;
};

/* Clamp a number into a range */
function clamp(n: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, n));
}

/* Parse a numeric input safely */
function parseNumber(value: string, fallback: number): number {
  const n = Number(value);
  return Number.isFinite(n) ? n : fallback;
}

/* Simple validation for config */
function validate(config: AnalysisConfig): string | null {
  const w = config.weights;
  const weights = [w.germination, w.growth, w.disease, w.aroma];
  if (weights.some((x) => !Number.isFinite(x) || x < 0)) {
    return "重みは 0 以上の数値にしてください";
  }
  const sum = weights.reduce((a, b) => a + b, 0);
  if (sum <= 0) {
    return "重みの合計は 0 より大きくしてください";
  }
  if (config.keep_threshold < config.review_threshold) {
    return "keep 閾値は review 閾値以上にしてください";
  }
  return null;
}

/* Analysis config modal */
export function AnalysisConfigModal({
  open,
  config,
  onClose,
  onChange,
  onApply,
}: Props): ReactElement | null {
  if (!open) return null;

  const error = validate(config);

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/70 backdrop-blur-sm">
      <div className="mx-auto flex h-full max-w-3xl items-center justify-center p-6">
        <div className="w-full rounded-2xl bg-slate-900/80 p-5 shadow-sm ring-1 ring-slate-800">
          <div className="flex items-start justify-between gap-3">
            <div>
              <div className="text-lg font-semibold text-slate-100">
                解析設定
              </div>
              <div className="mt-1 text-sm text-slate-400">
                重み・閾値を変更して再解析できます（0..100）
              </div>
            </div>
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg bg-slate-950/40 px-3 py-2 text-sm
                text-slate-200 ring-1 ring-slate-800 hover:bg-slate-900/60"
            >
              閉じる
            </button>
          </div>

          {error ? (
            <div
              className="mt-3 rounded-lg bg-rose-500/10 p-3 text-sm
                text-rose-200 ring-1 ring-rose-500/20"
            >
              {error}
            </div>
          ) : null}

          <div className="mt-4 grid grid-cols-12 gap-4">
            <div className="col-span-7 rounded-xl bg-slate-950/40 p-4 ring-1 ring-slate-800">
              <div className="text-sm font-semibold text-slate-200">重み</div>
              <div className="mt-3 grid grid-cols-2 gap-3">
                {(
                  [
                    ["germination", "発芽率"],
                    ["growth", "生育"],
                    ["disease", "耐病性"],
                    ["aroma", "香り"],
                  ] as const
                ).map(([key, label]) => (
                  <label key={key} className="flex flex-col gap-1">
                    <span className="text-xs text-slate-400">{label}</span>
                    <input
                      value={String(config.weights[key])}
                      onChange={(e) => {
                        const next = parseNumber(
                          e.currentTarget.value,
                          config.weights[key],
                        );
                        onChange({
                          ...config,
                          weights: { ...config.weights, [key]: next },
                        });
                      }}
                      className="w-full rounded-lg bg-slate-950/40 px-3 py-2
                        text-sm text-slate-100 ring-1 ring-slate-800"
                    />
                  </label>
                ))}
              </div>

              <div className="mt-3 flex items-center justify-between">
                <label className="flex items-center gap-2 text-sm text-slate-200">
                  <input
                    type="checkbox"
                    checked={config.normalize_to_100}
                    onChange={(e) =>
                      onChange({
                        ...config,
                        normalize_to_100: e.currentTarget.checked,
                      })
                    }
                  />
                  0..100 に正規化
                </label>
                <button
                  type="button"
                  onClick={() => onChange(defaultAnalysisConfig)}
                  className="rounded-lg bg-slate-950/40 px-3 py-2 text-sm
                    text-slate-200 ring-1 ring-slate-800 hover:bg-slate-900/60"
                >
                  デフォルトに戻す
                </button>
              </div>
            </div>

            <div className="col-span-5 rounded-xl bg-slate-950/40 p-4 ring-1 ring-slate-800">
              <div className="text-sm font-semibold text-slate-200">閾値</div>
              <div className="mt-3 grid gap-3">
                <label className="flex flex-col gap-1">
                  <span className="text-xs text-slate-400">keep</span>
                  <input
                    value={String(config.keep_threshold)}
                    onChange={(e) => {
                      const next = clamp(
                        parseNumber(e.currentTarget.value, config.keep_threshold),
                        0,
                        100,
                      );
                      onChange({ ...config, keep_threshold: next });
                    }}
                    className="w-full rounded-lg bg-slate-950/40 px-3 py-2
                      text-sm text-slate-100 ring-1 ring-slate-800"
                  />
                </label>
                <label className="flex flex-col gap-1">
                  <span className="text-xs text-slate-400">review</span>
                  <input
                    value={String(config.review_threshold)}
                    onChange={(e) => {
                      const next = clamp(
                        parseNumber(
                          e.currentTarget.value,
                          config.review_threshold,
                        ),
                        0,
                        100,
                      );
                      onChange({ ...config, review_threshold: next });
                    }}
                    className="w-full rounded-lg bg-slate-950/40 px-3 py-2
                      text-sm text-slate-100 ring-1 ring-slate-800"
                  />
                </label>
              </div>

              <div className="mt-4 flex justify-end gap-2">
                <button
                  type="button"
                  onClick={onClose}
                  className="rounded-lg bg-slate-950/40 px-4 py-2 text-sm
                    text-slate-200 ring-1 ring-slate-800 hover:bg-slate-900/60"
                >
                  キャンセル
                </button>
                <button
                  type="button"
                  onClick={onApply}
                  disabled={Boolean(error)}
                  className="rounded-lg bg-slate-800 px-4 py-2 text-sm
                    font-semibold text-slate-100 ring-1 ring-slate-700
                    hover:bg-slate-750 disabled:cursor-not-allowed
                    disabled:opacity-60"
                >
                  再解析
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

