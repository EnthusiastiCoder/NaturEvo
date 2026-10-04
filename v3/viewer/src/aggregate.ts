// Pure aggregation maths for the landing charts. Empty input never yields NaN.
export type Rate = { k: number; n: number; rate: number | null };
export const rateOf = (k: number, n: number): Rate => ({ k, n, rate: n > 0 ? k / n : null });

export function campaignWinRate(c: { on?: { wins: number; losses: number }; off?: { wins: number; losses: number } }): { on: Rate; off: Rate } {
  const f = (s?: { wins: number; losses: number }) => rateOf(s?.wins ?? 0, (s?.wins ?? 0) + (s?.losses ?? 0));
  return { on: f(c.on), off: f(c.off) };
}

export function poolRates(parts: { k: number; n: number }[]): Rate {
  return rateOf(parts.reduce((a, p) => a + p.k, 0), parts.reduce((a, p) => a + p.n, 0));
}

type CurveRow = Record<string, number | undefined>;
export function learningCurves(rows: CurveRow[]) {
  const col = (c: string) => rows.map((r) => r[c] ?? 0);
  const h = col('hof_wr_hero'), m = col('hof_wr_monster');
  return {
    gens: col('gen'), heroFitness: col('best_hero'), monsterFitness: col('best_monster'),
    heroWinVsEarlier: h, monsterWinVsEarlier: m,
    drawRate: rows.length ? h.map((v, i) => Math.max(0, 1 - v - m[i])) : null,
  };
}

export type Shares = { n: number; attack: number; block: number; dodge: number };
const shares = (xs: { action: string }[]): Shares => {
  const n = xs.length, c = (a: string) => (n ? xs.filter((x) => x.action === a).length / n : 0);
  return { n, attack: c('attack'), block: c('block'), dodge: c('dodge') };
};

export function actionShares(ticks: { action: string; seen?: boolean; dist?: number }[], opts: { distBuckets: number[] }) {
  if (ticks.some((t) => t.seen === undefined || t.dist === undefined)) return null;
  const b = opts.distBuckets;
  const bucket = (d: number) => { const i = b.findIndex((x) => d <= x); return i < 0 ? b.length : i; };
  const byDistance = Array.from({ length: b.length + 1 }, (_, i) => shares(ticks.filter((t) => bucket(t.dist!) === i)));
  return { byVisibility: { seen: shares(ticks.filter((t) => t.seen)), unseen: shares(ticks.filter((t) => !t.seen)) }, byDistance };
}

export function zoneRates(encs: { zone: number; result: string }[]): { zone: number; k: number; n: number; rate: number | null }[] {
  const z = new Map<number, { k: number; n: number }>();
  for (const e of encs) { const a = z.get(e.zone) ?? { k: 0, n: 0 }; a.n++; if (e.result === 'HeroWin') a.k++; z.set(e.zone, a); }
  return [...z].sort((a, b) => a[0] - b[0]).map(([zone, a]) => ({ zone, ...rateOf(a.k, a.n) }));
}
