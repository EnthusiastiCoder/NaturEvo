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
