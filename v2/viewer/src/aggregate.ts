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
