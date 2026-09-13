import { describe, it, expect } from 'vitest';
import { fightCamera, mapCamera, separate, stateLabel } from '../src/layout';

const ARENA = { w: 1536, h: 819 };
describe('fight camera', () => {
  it.each([[1282, 642], [1920, 1080]])('zooms so fighters are large at %ix%i and never shows outside the arena', (w, h) => {
    const c = fightCamera(ARENA, [700, 400], [760, 400], { w, h });
    expect(c.scale).toBeGreaterThanOrEqual(1.5);
    expect(c.x).toBeLessThanOrEqual(0);
    expect(c.y).toBeLessThanOrEqual(0);
    expect(c.x + ARENA.w * c.scale).toBeGreaterThanOrEqual(w - 1e-6);
    expect(c.y + ARENA.h * c.scale).toBeGreaterThanOrEqual(h - 1e-6);
  });
  it('keeps both fighters in view when far apart', () => {
    const st = { w: 1920, h: 1080 };
    const c = fightCamera(ARENA, [200, 200], [1300, 600], st);
    for (const p of [[200, 200], [1300, 600]]) {
      expect(p[0] * c.scale + c.x).toBeGreaterThanOrEqual(0);
      expect(p[0] * c.scale + c.x).toBeLessThanOrEqual(st.w);
      expect(p[1] * c.scale + c.y).toBeGreaterThanOrEqual(0);
      expect(p[1] * c.scale + c.y).toBeLessThanOrEqual(st.h);
    }
  });
});
describe('map camera', () => {
  it('zooms around the hero and clamps to the map', () => {
    const m = { w: 4000, h: 3000 };
    const c = mapCamera(m, [2000, 1500], { w: 1920, h: 1080 });
    expect(c.scale).toBeGreaterThanOrEqual(1.4);
    expect(2000 * c.scale + c.x).toBeCloseTo(960, 3);
    const edge = mapCamera(m, [0, 0], { w: 1920, h: 1080 });
    expect(edge.x).toBe(0); expect(edge.y).toBe(0);
  });
});
describe('visual separation', () => {
  it('pushes overlapping fighters apart symmetrically, leaves distant ones alone', () => {
    const [a, b] = separate([10, 10], [10.2, 10], 1.5);
    expect(Math.hypot(a[0] - b[0], a[1] - b[1])).toBeCloseTo(1.5, 6);
    expect((a[0] + b[0]) / 2).toBeCloseTo(10.1, 6);
    expect(separate([0, 0], [3, 0], 1.5)).toEqual([[0, 0], [3, 0]]);
  });
  it('handles exact overlap', () => {
    const [a, b] = separate([5, 5], [5, 5], 1);
    expect(Math.hypot(a[0] - b[0], a[1] - b[1])).toBeCloseTo(1, 6);
  });
});
describe('state labels', () => {
  it('gives attack, block and dodge distinct readable labels', () => {
