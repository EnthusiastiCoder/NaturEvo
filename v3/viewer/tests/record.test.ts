// Assumed module: src/record.ts exports planFilmSequence(maxGen: number): {gen:number, caption:string}[]
// The learning film shows gens 0, 10, 30, 80, 200, limited to gens <= maxGen.
import { describe, it, expect } from 'vitest';
import { planFilmSequence } from '../src/record';

describe('planFilmSequence', () => {
  it('plans gens 0,10,30,80,200 for a 200-gen run', () => {
    const s = planFilmSequence(200);
    expect(s.map((x) => x.gen)).toEqual([0, 10, 30, 80, 200]);
    for (const x of s) {
      expect(x.caption).toContain(String(x.gen));
      expect(x.caption.length).toBeGreaterThan(0);
    }
  });
  it('drops gens beyond the run length', () => {
    expect(planFilmSequence(100).map((x) => x.gen)).toEqual([0, 10, 30, 80]);
  });
  it('a run of 0 gens still plans gen 0', () => {
    expect(planFilmSequence(0).map((x) => x.gen)).toEqual([0]);
  });
});
