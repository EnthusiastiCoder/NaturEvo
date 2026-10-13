// Pure impact-effect maths for the fight view. Everything is a function of tick age, so scrubbing and pause stay deterministic.
import type { Seg } from './data';

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));

export interface HitFx { scale: number; sparks: number; ringR: number; flashR: number; shake: number }
/** Effect sizes for a hit of the given impact (sim impact is ~200 for a full blow). */
export function hitFx(impact: number): HitFx {
  const scale = clamp(impact / 200, 0.3, 1.8);
  return { scale, sparks: Math.round(clamp(12 + 58 * scale, 12, 120)), ringR: 40 + 50 * scale, flashR: 18 + 30 * scale, shake: clamp(2 + 6 * scale, 2, 14) };
}
