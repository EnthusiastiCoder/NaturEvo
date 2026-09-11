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
