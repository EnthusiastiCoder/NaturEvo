// Assumed module: src/aggregate.ts
//   aggregateSeries(seeds: number[][]): {gens:number[], mean:number[], min:number[], max:number[]}
//     one array per seed, index = generation; mean and min-max band across seeds
//   aggregateEval(evals: {defence_rate,idle_guard,draw_share,win_rate_hof}[]): per-metric {mean,min,max}
import { describe, it, expect } from 'vitest';
import { aggregateSeries, aggregateEval } from '../src/aggregate';

describe('aggregateSeries', () => {
  it('computes mean and min-max band across 3 seeds', () => {
    const a = aggregateSeries([[1, 4], [2, 5], [3, 9]]);
    expect(a.gens).toEqual([0, 1]);
    expect(a.mean[0]).toBeCloseTo(2, 9);
    expect(a.min[0]).toBe(1);
    expect(a.max[0]).toBe(3);
    expect(a.mean[1]).toBeCloseTo(6, 9);
    expect(a.min[1]).toBe(4);
    expect(a.max[1]).toBe(9);
  });
  it('empty input gives empty series, never NaN', () => {
    const a = aggregateSeries([]);
    expect(a.gens).toEqual([]);
    expect(a.mean).toEqual([]);
  });
});
