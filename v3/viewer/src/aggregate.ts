// Pure aggregation maths for the landing charts. Empty input never yields NaN.
export type Rate = { k: number; n: number; rate: number | null };
export const rateOf = (k: number, n: number): Rate => ({ k, n, rate: n > 0 ? k / n : null });

export function campaignWinRate(c: { on?: { wins: number; losses: number }; off?: { wins: number; losses: number } }): { on: Rate; off: Rate } {
  const f = (s?: { wins: number; losses: number }) => rateOf(s?.wins ?? 0, (s?.wins ?? 0) + (s?.losses ?? 0));
  return { on: f(c.on), off: f(c.off) };
}
