import { describe, it, expect } from 'vitest';
import { displayPlan, fitCamera, hudOffsets, uiScale, separate, VISUAL_MIN_DIST, fightCamera } from '../src/layout';
import { fightStats, statsLine, type Frame } from '../src/interp';
import { lowResSub } from '../src/ground';

describe('crisp rendering', () => {
  it('renderer backing store matches displayed device pixels (no css downscale)', () => {
    const p = displayPlan(1301, 732, 1920, 1080, 1.25);
    expect(p.resolution).toBe(1.25);
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
  it('status row sits between the label and the bar, scaling linearly with u', () => {
    for (const u of [1, 2.5]) {
      const o = hudOffsets(u) as ReturnType<typeof hudOffsets> & { statusRowTop: number; statusRowBottom: number };
      expect(o.labelBottom).toBeLessThanOrEqual(o.statusRowTop);
      expect(o.statusRowTop).toBeLessThan(o.statusRowBottom);
      expect(o.statusRowBottom).toBeLessThan(o.barTop);
    }
    const a = hudOffsets(1) as any, b = hudOffsets(2.5) as any;
    expect(b.statusRowTop).toBeCloseTo(a.statusRowTop * 2.5, 6);
    expect(b.statusRowBottom).toBeCloseTo(a.statusRowBottom * 2.5, 6);
  });
  it('fighters are never smaller than a readable size when the camera is zoomed out', () => {
    expect(uiScale(2)).toBe(1);
    expect(uiScale(0.64) * 0.64).toBeGreaterThanOrEqual(1.5 - 1e-9);
  });
  it('record camera starts close on the fight', () => {
    const arena = { w: 1536, h: 1024 };
    const c = fightCamera(arena, [200, 400], [1200, 400], { w: 1920, h: 1080 });
    expect(1000 * c.scale).toBeLessThanOrEqual(1920); // both fighters still in view
    expect(c.scale).toBeGreaterThan(1.3);
    expect(fightCamera(arena, [700, 400], [800, 400], { w: 1920, h: 1080 }).scale).toBeGreaterThan(2);
  });
});

describe('split-screen fit', () => {
  it('fits the whole arena inside a half and centres it', () => {
    const arena = { w: 1536, h: 819 };
    const c = fitCamera(arena, { w: 960, h: 1080 });
    expect(arena.w * c.scale).toBeLessThanOrEqual(960 + 1e-6);
    expect(arena.h * c.scale).toBeLessThanOrEqual(1080 + 1e-6);
    expect(c.x + arena.w * c.scale / 2).toBeCloseTo(480);
    expect(c.y + arena.h * c.scale / 2).toBeCloseTo(540);
  });
});

const fr = (hh: number, mh: number, hs: string, ms: string): Frame => ({
  hero: { pos: [0, 0], facing: 0, hp: hh, stamina: 100, action: 'none', state: hs },
  monster: { pos: [1, 0], facing: 0, hp: mh, stamina: 100, action: 'none', state: ms },
});
describe('fight stats strip', () => {
  const frames = [fr(100, 100, 'Idle', 'Idle'), fr(100, 90, 'Active', 'Idle'), fr(100, 90, 'Idle', 'Blocking'), fr(100, 90, 'Idle', 'Blocking'),
    fr(80, 90, 'Idle', 'Active'), fr(80, 90, 'Dodging', 'Idle'), fr(80, 70, 'Idle', 'Idle')];
  it('counts hits landed, blocks and dodges per fighter up to a frame', () => {
    const s = fightStats(frames, frames.length - 1);
    expect(s.hero).toEqual({ hits: 2, blocks: 0, dodges: 1 });
    expect(s.monster).toEqual({ hits: 1, blocks: 1, dodges: 0 });
    expect(fightStats(frames, 1).hero.hits).toBe(1);
    expect(fightStats(frames, 0).hero.hits).toBe(0);
  });
  it('formats a short line', () => {
    expect(statsLine('Hero', { hits: 1, blocks: 2, dodges: 3 })).toBe('Hero: 1 hits, 2 blocks, 3 dodges');
  });
});
