// Assumed module: src/aggregate.ts (see v2/tests-index-viewer-landing.md)
import { describe, it, expect } from 'vitest';
import { campaignWinRate, poolRates, learningCurves, actionShares, zoneRates, statusesPerFight } from '../src/aggregate';

describe('campaignWinRate', () => {
  it('returns counts and rate for on vs off', () => {
    const r = campaignWinRate({ on: { wins: 1, losses: 3 }, off: { wins: 6, losses: 2 } });
    expect(r.on).toEqual({ k: 1, n: 4, rate: 0.25 });
    expect(r.off).toEqual({ k: 6, n: 8, rate: 0.75 });
  });
  it('missing or zero-count side gives rate null, never NaN', () => {
    const r = campaignWinRate({ on: { wins: 0, losses: 0 } });
    expect(r.on).toEqual({ k: 0, n: 0, rate: null });
    expect(r.off).toEqual({ k: 0, n: 0, rate: null });
  });
});

describe('poolRates', () => {
  it('weights by counts, not by mean of means', () => {
    // 1/1 and 1/9 -> 2/10 = 0.2 (mean of means would be 0.55)
    expect(poolRates([{ k: 1, n: 1 }, { k: 1, n: 9 }])).toEqual({ k: 2, n: 10, rate: 0.2 });
  });
  it('empty or all-zero -> rate null', () => {
    expect(poolRates([])).toEqual({ k: 0, n: 0, rate: null });
    expect(poolRates([{ k: 0, n: 0 }])).toEqual({ k: 0, n: 0, rate: null });
  });
});

describe('learningCurves', () => {
  const rows = [
    { gen: 0, best_hero: 0.5, best_monster: 0.25, hof_wr_hero: 0, hof_wr_monster: 0 },
    { gen: 1, best_hero: 2.5, best_monster: 1.5, hof_wr_hero: 0.4, hof_wr_monster: 0.6 },
    { gen: 2, best_hero: 3, best_monster: 2, hof_wr_hero: 0.5, hof_wr_monster: 0.25 },
  ] as any;
  it('series have one point per generation', () => {
    const c = learningCurves(rows);
    expect(c.gens).toEqual([0, 1, 2]);
    expect(c.heroFitness).toEqual([0.5, 2.5, 3]);
    expect(c.monsterFitness).toEqual([0.25, 1.5, 2]);
    expect(c.heroWinVsEarlier).toEqual([0, 0.4, 0.5]);
    expect(c.monsterWinVsEarlier).toEqual([0, 0.6, 0.25]);
  });
  it('draw rate = 1 - hero - monster per generation', () => {
    expect(learningCurves(rows).drawRate).toEqual([1, 0, 0.25]);
  });
  it('empty input gives empty series and no draw rate, no NaN', () => {
    const c = learningCurves([]);
    expect(c.gens).toEqual([]);
    expect(c.heroFitness).toEqual([]);
    expect(c.drawRate).toBeNull();
  });
});

describe('actionShares', () => {
  const ticks = [
    { action: 'attack', seen: true, dist: 1 },
    { action: 'attack', seen: true, dist: 1 },
    { action: 'block', seen: true, dist: 5 },
    { action: 'dodge', seen: false, dist: 5 },
  ] as any;
  it('by visibility', () => {
    const s = actionShares(ticks, { distBuckets: [3] })!;
    expect(s.byVisibility.seen).toEqual({ n: 3, attack: 2 / 3, block: 1 / 3, dodge: 0 });
    expect(s.byVisibility.unseen).toEqual({ n: 1, attack: 0, block: 0, dodge: 1 });
  });
  it('by distance bucket: <=3 near, >3 far', () => {
    const s = actionShares(ticks, { distBuckets: [3] })!;
    expect(s.byDistance.map(b => b.n)).toEqual([2, 2]);
    expect(s.byDistance[0]).toMatchObject({ attack: 1, block: 0, dodge: 0 });
    expect(s.byDistance[1]).toMatchObject({ attack: 0, block: 0.5, dodge: 0.5 });
  });
  it('no ticks -> n 0 and zero shares, no NaN', () => {
    const s = actionShares([], { distBuckets: [3] })!;
    expect(s.byVisibility.seen).toEqual({ n: 0, attack: 0, block: 0, dodge: 0 });
    expect(s.byVisibility.unseen.n).toBe(0);
  });
  it('returns null when data lacks vision fields', () => {
    expect(actionShares([{ action: 'attack' }] as any, { distBuckets: [3] })).toBeNull();
  });
});

describe('zoneRates', () => {
  it('per-zone win rate with counts, sorted by zone', () => {
    const z = zoneRates([
      { zone: 2, result: 'HeroWin' }, { zone: 1, result: 'MonsterWin' },
      { zone: 1, result: 'HeroWin' }, { zone: 1, result: 'HeroWin' }, { zone: 2, result: 'MonsterWin' },
    ]);
    expect(z).toEqual([
      { zone: 1, k: 2, n: 3, rate: 2 / 3 },
      { zone: 2, k: 1, n: 2, rate: 0.5 },
    ]);
  });
  it('empty -> []', () => expect(zoneRates([])).toEqual([]));
});
