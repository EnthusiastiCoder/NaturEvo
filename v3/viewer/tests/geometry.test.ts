import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight, DEFAULT_GEOMETRY } from '../src/data';
import { shieldPlate } from '../src/robot-draw';

const small = () => JSON.parse(readFileSync(resolve(__dirname, 'fixtures/fight_small.json'), 'utf8'));
const withBlocks = () => ({
  ...small(),
  arena: { width: 14, height: 6, obstacles: [], biome: 'jungle' },
  geometry: { head_r: 23, neck_r: 9, shield_len: 70, shield_along: 62, shield_r: 5, shield_orient: 'perpendicular', sword_len: 220, sword_r: 5, radii: [23, 30, 25, 14, 14, 12, 12, 17, 17, 12, 12, 10, 10] },
  sudden_death: { start_tick: 450, speed: 0.6, min_width: 1.6 }, // legacy block: must be ignored
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
  });
  it('ignores malformed blocks', () => {
    const j = withBlocks(); (j as any).geometry = { head_r: 'x', radii: [1] };
    const f = parseFight(j);
    expect(f.geometry).toEqual(DEFAULT_GEOMETRY);
  });
  it('ignores a legacy sudden_death block', () => {
    expect(parseFight(withBlocks())).not.toHaveProperty('suddenDeath');
  });
  it('reads result.capped', () => {
    expect(parseFight({ ...withBlocks(), result: { outcome: 'draw', ticks: 9, defence: [], capped: true } }).result.capped).toBe(true);
  });
});

describe('shield plate from geometry', () => {
  it('uses logged length and position', () => {
    const fa: [number, number, number, number] = [0, 0, 100, 0];
    const s = shieldPlate(fa, 0.5, 40);
    expect(s).toEqual([50, -20, 50, 20]);
  });
});

import { blockPlates } from '../src/robot-draw';
describe('blockPlates keeps the plate a block just destroyed', () => {
  const fa = [100, 0, 100, 80] as [number, number, number, number];
  const mk = (hp5: number) => ({ parts: Array.from({ length: 13 }, () => fa), hp: Array.from({ length: 13 }, (_, i) => (i === 5 ? hp5 : 100)) }) as never;
  it('uses hp before the event so the plate lost in the same tick still counts', () => {
    expect(blockPlates([mk(0)], [mk(41)], 0.62, 70)).toHaveLength(1);
  });
  it('skips plates already gone before the event', () => {
    expect(blockPlates([mk(0)], [mk(0)], 0.62, 70)).toHaveLength(0);
  });
});
