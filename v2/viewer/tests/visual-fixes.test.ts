import { describe, it, expect } from 'vitest';
import { displayPlan, fitCamera, hudOffsets, uiScale, separate, VISUAL_MIN_DIST, fightCamera } from '../src/layout';
import { fightStats, statsLine, type Frame } from '../src/interp';
import { lowResSub } from '../src/ground';

describe('crisp rendering', () => {
  it('renderer backing store matches displayed device pixels (no css downscale)', () => {
    const p = displayPlan(1301, 732, 1920, 1080, 1.25);
    expect(p.resolution).toBe(1.25);
    expect(p.cssW * p.stageScale).toBeCloseTo(p.cssW);
    expect(p.stageScale * 1920).toBeCloseTo(p.cssW, 5);
    expect(p.cssW / p.cssH).toBeCloseTo(16 / 9, 1);
  });
  it('ground colour/noise is sampled at least every 2 world px', () => {
    expect(32 / lowResSub(32)).toBeLessThanOrEqual(2);
    expect(48 / lowResSub(48)).toBeLessThanOrEqual(2);
  });
});
