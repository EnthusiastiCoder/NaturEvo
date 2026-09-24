// Pure aggregation maths for the landing charts. Empty input never yields NaN.
export type Rate = { k: number; n: number; rate: number | null };
export const rateOf = (k: number, n: number): Rate => ({ k, n, rate: n > 0 ? k / n : null });
