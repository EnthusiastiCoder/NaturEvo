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
