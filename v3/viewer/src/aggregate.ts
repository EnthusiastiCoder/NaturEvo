// Pure aggregation maths for the landing charts. Empty input never yields NaN.
export type Band = { gens: number[]; mean: number[]; min: number[]; max: number[] };
export function aggregateSeries(seeds: number[][]): Band {
  const n = seeds.length ? Math.min(...seeds.map((s) => s.length)) : 0;
  const out: Band = { gens: [], mean: [], min: [], max: [] };
  for (let i = 0; i < n; i++) {
    const v = seeds.map((s) => s[i]);
    out.gens.push(i);
    out.mean.push(v.reduce((a, b) => a + b, 0) / v.length);
    out.min.push(Math.min(...v));
    out.max.push(Math.max(...v));
  }
  return out;
}

export const EVAL_KEYS = ['defence_rate', 'idle_guard', 'draw_share', 'win_rate_hof'] as const;
export type EvalKey = (typeof EVAL_KEYS)[number];
export type EvalRow = Record<EvalKey, number>;
export type Range = { mean: number; min: number; max: number };
export function aggregateEval(evals: EvalRow[]): Record<EvalKey, Range> {
  const out = {} as Record<EvalKey, Range>;
  for (const k of EVAL_KEYS) {
    const v = evals.map((e) => e[k]).filter((x) => Number.isFinite(x));
    out[k] = v.length ? { mean: v.reduce((a, b) => a + b, 0) / v.length, min: Math.min(...v), max: Math.max(...v) } : { mean: 0, min: 0, max: 0 };
  }
  return out;
}

/** Parry/block/dodge shares of all successful defences in a fight result (both fighters). */
export function defenceShares(def: { parry?: number; block?: number; dodge?: number }[]): { parry: number; block: number; dodge: number } {
  const s = (k: 'parry' | 'block' | 'dodge') => def.reduce((a, d) => a + (d[k] ?? 0), 0);
  const p = s('parry'), b = s('block'), d = s('dodge'), t = p + b + d;
  return t > 0 ? { parry: p / t, block: b / t, dodge: d / t } : { parry: 0, block: 0, dodge: 0 };
}

/** Centred moving average; the window shrinks at the edges so the length is unchanged. */
export function smooth(ys: number[], w: number): number[] {
  const h = Math.floor(w / 2);
  return ys.map((_, i) => {
    const a = Math.max(0, i - h), b = Math.min(ys.length - 1, i + h);
    let s = 0;
    for (let j = a; j <= b; j++) s += ys[j];
    return s / (b - a + 1);
  });
}
