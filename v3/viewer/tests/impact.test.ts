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
