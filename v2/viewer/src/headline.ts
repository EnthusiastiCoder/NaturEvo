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

export function headline(data: { run: RunInfo; rows: Row[] }[]): Headline {
  const on = side(data.map((d) => d.run), 'on'), off = side(data.map((d) => d.run), 'off');
  const m = (c: string) => mean(data.map((d) => tail(d.rows, c)));
  const pair = (a: string, b: string) => { const x = m(a), y = m(b); return x === null || y === null ? null : { hero: x, monster: y }; };
  const drawCol = m('draw_rate');
  const lve = pair('hof_wr_hero', 'hof_wr_monster');
  return {
    campaign: on.n + off.n > 0 ? { on, off } : null,
    lateVsEarly: lve,
    draw: drawCol ?? (lve ? Math.max(0, 1 - lve.hero - lve.monster) : null),
    unseenBlock: pair('block_unseen_share_hero', 'block_unseen_share_monster'),
  };
}
