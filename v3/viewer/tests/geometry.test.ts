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
describe('blockPlates: a plate exists until it is destroyed (detached), not until hp reaches 0', () => {
  const fa = [100, 0, 100, 80] as [number, number, number, number];
  const mk = () => ({ parts: Array.from({ length: 13 }, () => fa) }) as never;
  it('keeps a plate that is only disabled (hp 0, not yet destroyed): the sim still blocks with it', () => {
    expect(blockPlates([mk(), mk()], [false, false], 0.62, 70)).toHaveLength(2);
  });
  it('skips a plate already detached before the event', () => {
    expect(blockPlates([mk(), mk()], [false, true], 0.62, 70)).toHaveLength(1);
  });
});
