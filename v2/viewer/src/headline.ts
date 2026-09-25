// The few numbers that answer "how is this run doing" at a glance.
import { type Rate, poolRates, tailMean } from './aggregate';
import type { Row, RunInfo } from './runs-index';

export type Headline = {
  campaign: { on: Rate; off: Rate } | null;
  lateVsEarly: { hero: number; monster: number } | null;
  draw: number | null;
  unseenBlock: { hero: number; monster: number } | null;
};
const TAIL = 10; // last generations averaged

const side = (runs: RunInfo[], s: 'on' | 'off') => poolRates(runs.map((r) => { const c = r.permadeath[s]; return c ? { k: c.wins, n: c.wins + c.losses } : { k: 0, n: 0 }; }));
function mean(xs: (number | null)[]): number | null {
  const v = xs.filter((x): x is number => x !== null);
  return v.length ? v.reduce((a, b) => a + b, 0) / v.length : null;
}
const tail = (rows: Row[], c: string) => (rows.some((r) => r[c] !== undefined) ? tailMean(rows.map((r) => r[c] ?? 0), TAIL) : null);
