// Pure impact-effect maths for the fight view. Everything is a function of tick age, so scrubbing and pause stay deterministic.
import type { Seg } from './data';

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));

export interface HitFx { scale: number; sparks: number; ringR: number; flashR: number; shake: number }
/** Effect sizes for a hit of the given impact (sim impact is ~200 for a full blow). */
export function hitFx(impact: number): HitFx {
  const scale = clamp(impact / 200, 0.3, 1.8);
  return { scale, sparks: Math.round(clamp(12 + 58 * scale, 12, 120)), ringR: 40 + 50 * scale, flashR: 18 + 30 * scale, shake: clamp(2 + 6 * scale, 2, 14) };
}

export interface HitstopPlan { freeze: number; slow: number; slowFactor: number }
/** Ticks of full freeze, then ticks of slow-motion, for an event's hitstop_ticks. Heavy stops get a slow tail. */
export function hitstopPlan(ticks: number, reduced: boolean): HitstopPlan {
  if (reduced || ticks <= 0) return { freeze: 0, slow: 0, slowFactor: 1 };
  const freeze = Math.min(10, ticks);
  return ticks >= 5 ? { freeze, slow: 8, slowFactor: 0.35 } : { freeze, slow: 0, slowFactor: 1 };
}
