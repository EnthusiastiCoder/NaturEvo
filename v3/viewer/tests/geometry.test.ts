import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight, DEFAULT_GEOMETRY } from '../src/data';
import { wallsAt, bleedAt, suddenDeathActive } from '../src/walls';
import { shieldPlate } from '../src/robot-draw';

const small = () => JSON.parse(readFileSync(resolve(__dirname, 'fixtures/fight_small.json'), 'utf8'));
const withBlocks = () => ({
  ...small(),
  arena: { width: 14, height: 6, obstacles: [], biome: 'jungle' },
  geometry: { head_r: 23, neck_r: 9, shield_len: 70, shield_along: 62, shield_r: 5, shield_orient: 'perpendicular', sword_len: 220, sword_r: 5, radii: [23, 30, 25, 14, 14, 12, 12, 17, 17, 12, 12, 10, 10] },
  sudden_death: { start_tick: 450, speed: 0.6, min_width: 1.6 },
});

describe('parseFight geometry', () => {
  it('reads geometry in cm and shield fraction', () => {
    const g = parseFight(withBlocks()).geometry;
    expect(g.headR).toBe(23); expect(g.neckR).toBe(9);
    expect(g.shieldLen).toBe(70); expect(g.shieldAlong).toBeCloseTo(0.62);
    expect(g.radii[2]).toBe(25); expect(g.radii).toHaveLength(13);
  });
  it('falls back to current constants for old logs', () => {
    const f = parseFight(small());
    expect(f.geometry).toEqual(DEFAULT_GEOMETRY);
    expect(f.suddenDeath).toBeNull();
  });
  it('ignores malformed blocks', () => {
    const j = withBlocks(); (j as any).geometry = { head_r: 'x', radii: [1] }; (j as any).sudden_death = { start_tick: 'a' };
    const f = parseFight(j);
    expect(f.geometry).toEqual(DEFAULT_GEOMETRY); expect(f.suddenDeath).toBeNull();
  });
  it('reads sudden death with min width in cm', () => {
    expect(parseFight(withBlocks()).suddenDeath).toMatchObject({ startTick: 450, speedCm: 60, minWidth: 160 });
  });
});

describe('walls', () => {
  const f = parseFight(withBlocks());
  it('sit 0.5 m inside the arena until sudden death', () => {
    expect(wallsAt(f, 0)).toEqual({ lo: 50, hi: 1350 });
    expect(wallsAt(f, 450)).toEqual({ lo: 50, hi: 1350 });
  });
  it('close symmetrically at 0.6 m/s after the start tick', () => {
    const w = wallsAt(f, 450 + 60);
    expect(w.lo).toBeCloseTo(110); expect(w.hi).toBeCloseTo(1290);
  });
  it('stop at the minimum width 160 cm', () => {
    const w = wallsAt(f, 6000);
    expect(w.hi - w.lo).toBeCloseTo(160); expect((w.lo + w.hi) / 2).toBeCloseTo(700);
  });
  it('are static without a sudden_death block', () => {
    const o = parseFight({ ...withBlocks(), sudden_death: undefined });
    expect(wallsAt(o, 900)).toEqual({ lo: 50, hi: 1350 });
    expect(suddenDeathActive(o, 900)).toBe(false);
  });
  it('active flag and bleed rate', () => {
    expect(suddenDeathActive(f, 449)).toBe(false);
    expect(suddenDeathActive(f, 451)).toBe(true);
    expect(bleedAt(f, 100)).toBe(0);
    expect(bleedAt(f, 900)).toBeGreaterThan(bleedAt(f, 500));
  });
});
