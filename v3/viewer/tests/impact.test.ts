import { describe, it, expect } from 'vitest';
import { hitFx, hitstopPlan, shakeOffset, recoilCm, floatNumber, partFlashAt, blockPoint } from '../src/impact';
import { partDamageStyle } from '../src/damage-style';
import { sparkSpec, shakeFor } from '../src/fx';

describe('hitFx scales with impact', () => {
  it('heavier hits give more sparks, bigger ring and flash, stronger shake', () => {
    const lo = hitFx(40), hi = hitFx(200), big = hitFx(400);
    expect(hi.sparks).toBeGreaterThan(lo.sparks);
    expect(hi.ringR).toBeGreaterThan(lo.ringR);
    expect(hi.flashR).toBeGreaterThan(lo.flashR);
    expect(hi.shake).toBeGreaterThan(lo.shake);
    expect(big.scale).toBeGreaterThanOrEqual(hi.scale);
  });
  it('is bounded and never zero for a real hit', () => {
    expect(hitFx(0).sparks).toBeGreaterThan(5);
    expect(hitFx(1e6).sparks).toBeLessThanOrEqual(120);
    expect(hitFx(1e6).shake).toBeLessThanOrEqual(14);
  });
  it('hit cone is much stronger than before (200 impact gives at least 50 sparks)', () => {
    expect((sparkSpec({ t: 1, kind: 'hit', impact: 200, dx: 1, dy: 0 } as any) as any).count).toBeGreaterThanOrEqual(50);
    expect(shakeFor({ t: 1, kind: 'hit', impact: 200 } as any)).toBeGreaterThanOrEqual(6);
  });
});

describe('hitstopPlan', () => {
  it('maps event ticks to a freeze, plus slow-mo for heavy stops', () => {
    expect(hitstopPlan(0, false)).toEqual({ freeze: 0, slow: 0, slowFactor: 1 });
    const p = hitstopPlan(6, false);
    expect(p.freeze).toBeGreaterThanOrEqual(5);
    expect(p.slow).toBeGreaterThan(0);
    expect(p.slowFactor).toBeLessThan(1);
    expect(hitstopPlan(2, false).slow).toBe(0);
  });
  it('is capped and disabled under reduced motion', () => {
    expect(hitstopPlan(100, false).freeze).toBeLessThanOrEqual(10);
    expect(hitstopPlan(6, true)).toEqual({ freeze: 0, slow: 0, slowFactor: 1 });
  });
});

describe('shake / recoil / floating number are deterministic and decay', () => {
  it('shakeOffset is pure, decays to zero, and is zero when reduced', () => {
    expect(shakeOffset(2, 10, 17)).toEqual(shakeOffset(2, 10, 17));
    const [x0, y0] = shakeOffset(1, 10, 17);
    expect(Math.hypot(x0, y0)).toBeGreaterThan(0);
    expect(shakeOffset(40, 10, 17)).toEqual([0, 0]);
    expect(shakeOffset(1, 0, 17)).toEqual([0, 0]);
  });
  it('recoil peaks early, returns to zero, scales with impact', () => {
    expect(recoilCm(0, 200)).toBe(0);
    expect(recoilCm(3, 200)).toBeGreaterThan(recoilCm(3, 40));
    expect(recoilCm(3, 200)).toBeGreaterThan(recoilCm(14, 200));
    expect(recoilCm(30, 200)).toBe(0);
    expect(recoilCm(-1, 200)).toBe(0);
  });
  it('floating number rises and fades, absent before and after its life', () => {
    expect(floatNumber(-1)).toBeNull();
    const a = floatNumber(5)!, b = floatNumber(30)!;
    expect(b.rise).toBeGreaterThan(a.rise);
    expect(b.alpha).toBeLessThan(a.alpha);
    expect(floatNumber(200)).toBeNull();
  });
  it('part flash is white at impact then fades within ~8 ticks', () => {
    expect(partFlashAt(0)).toBe(1);
    expect(partFlashAt(4)).toBeCloseTo(0.5, 5);
    expect(partFlashAt(9)).toBe(0);
    expect(partFlashAt(-1)).toBe(0);
  });
});

describe('blockPoint snaps the spark onto the nearest shield plate', () => {
  it('returns the closest point on a plate segment', () => {
    const p = blockPoint(5, 10, [[0, 0, 0, 20]]);
    expect(p[0]).toBeCloseTo(0, 6); expect(p[1]).toBeCloseTo(10, 6);
  });
  it('clamps to the endpoint and passes through with no plates', () => {
    expect(blockPoint(5, 30, [[0, 0, 0, 20]])).toEqual([0, 20]);
    expect(blockPoint(5, 5, [])).toEqual([5, 5]);
  });
});
