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

describe('aggregateEval', () => {
  it('gives mean and range per metric', () => {
    const r: any = aggregateEval([
      { defence_rate: 0.2, idle_guard: 0.05, draw_share: 0.1, win_rate_hof: 0.5 },
      { defence_rate: 0.3, idle_guard: 0.07, draw_share: 0.0, win_rate_hof: 0.6 },
      { defence_rate: 0.4, idle_guard: 0.03, draw_share: 0.2, win_rate_hof: 0.4 },
    ] as any);
    expect(r.defence_rate.mean).toBeCloseTo(0.3, 9);
    expect(r.defence_rate.min).toBe(0.2);
    expect(r.defence_rate.max).toBe(0.4);
    expect(r.idle_guard.min).toBe(0.03);
    expect(r.draw_share.max).toBe(0.2);
  });
});
