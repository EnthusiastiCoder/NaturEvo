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

describe('melee readability', () => {
  it('visually separates close fighters by at least ~2 tiles', () => {
    expect(VISUAL_MIN_DIST).toBeGreaterThanOrEqual(2);
    const [a, b] = separate([10, 10], [10.3, 10], VISUAL_MIN_DIST);
    expect(Math.hypot(b[0] - a[0], b[1] - a[1])).toBeCloseTo(VISUAL_MIN_DIST);
  });
  it('state label sits above the HP bar, never over it', () => {
    for (const u of [1, 2.5]) {
      const o = hudOffsets(u);
      expect(o.labelBottom).toBeLessThan(o.barTop - 2 * u); // y grows down
    }
  });
  it('fighters are never smaller than a readable size when the camera is zoomed out', () => {
    expect(uiScale(2)).toBe(1);
    expect(uiScale(0.64) * 0.64).toBeGreaterThanOrEqual(1.5 - 1e-9);
  });
  it('record camera starts close on the fight', () => {
    const arena = { w: 1536, h: 1024 };
    const c = fightCamera(arena, [200, 400], [1200, 400], { w: 1920, h: 1080 });
    expect(1000 * c.scale).toBeLessThanOrEqual(1920); // both fighters still in view
    expect(c.scale).toBeGreaterThan(1.5);
  });
});
