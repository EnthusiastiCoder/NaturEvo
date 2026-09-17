// parseFight with and without the combat-depth log fields; demoFight coverage.
import { describe, it, expect } from 'vitest';
import { parseFight, demoFight } from '../src/data';

const base = (extra: Record<string, unknown> = {}, frameExtra: Record<string, unknown> = {}) => ({
  tick_hz: 30, seed: 7, monster_index: 2,
  arena: { w: 48, h: 32, blocked: null },
  frames: [
    { pos: [[1, 2], [3, 4]], facing: [0, 3], hp: [100, 90], stamina: [50, 60], action: ['none', 'none'], state: ['Idle', 'Windup'], ...frameExtra },
  ],
  ...extra,
});
