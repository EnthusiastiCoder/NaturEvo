// Assumed module: src/aggregate.ts
//   aggregateSeries(seeds: number[][]): {gens:number[], mean:number[], min:number[], max:number[]}
//     one array per seed, index = generation; mean and min-max band across seeds
//   aggregateEval(evals: {defence_rate,idle_guard,draw_share,win_rate_hof}[]): per-metric {mean,min,max}
import { describe, it, expect } from 'vitest';
import { aggregateSeries, aggregateEval } from '../src/aggregate';
