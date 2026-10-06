import type { Frame, Tally } from './data';

/** Four tiers: >=75 healthy, 50-74 worn, 1-49 critical, 0 disabled. */
export function partHealthColor(hp: number): string {
  if (hp >= 75) return '#3fb950';
  if (hp >= 50) return '#d4b41c';
  if (hp >= 1) return '#e0701a';
  return '#7a2f2f';
}
