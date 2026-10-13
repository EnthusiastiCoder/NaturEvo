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

const SHAKE_TICKS = 16;
/** Camera shake offset in px at `age` ticks after the event; deterministic, decays to zero. */
export function shakeOffset(age: number, amp: number, seed: number): [number, number] {
  if (amp <= 0 || age < 0 || age >= SHAKE_TICKS) return [0, 0];
  const d = (1 - age / SHAKE_TICKS) ** 2 * amp, ph = seed * 1.7 + age * 2.3;
  return [Math.sin(ph * 3.1) * d, Math.cos(ph * 2.7) * d * 0.7];
}

const RECOIL_TICKS = 18;
/** Recoil distance (cm) of the struck body along the hit direction: quick rise, eased return. */
export function recoilCm(age: number, impact: number): number {
  if (age <= 0 || age >= RECOIL_TICKS) return 0;
  const peak = 6 + 16 * clamp(impact / 200, 0.2, 1.6);
  const k = age < 3 ? age / 3 : 1 - (age - 3) / (RECOIL_TICKS - 3);
  return peak * k * k;
}

export const NUMBER_TICKS = 60;
/** Floating damage number: rise in cm and alpha at `age` ticks, null outside its life. */
export function floatNumber(age: number): { rise: number; alpha: number } | null {
  if (age < 0 || age >= NUMBER_TICKS) return null;
  const k = age / NUMBER_TICKS;
  return { rise: 70 * (1 - (1 - k) ** 2), alpha: k < 0.3 ? 1 : 1 - (k - 0.3) / 0.7 };
}
