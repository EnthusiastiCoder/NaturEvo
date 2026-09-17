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

describe('parseFight old logs', () => {
  const f = parseFight(base());
  it('defaults new per-fighter fields', () => {
    for (const k of ['hero', 'monster'] as const) {
      expect(f.frames[0][k].atk).toBe(-1);
      expect(f.frames[0][k].fx).toEqual([0, 0, 0, 0, 0, 0, 0]);
      expect(f.frames[0][k].seen).toBe(false);
    }
    expect(f.frames[0].ev).toEqual([]);
  });
  it('defaults fight-level fields to null and keeps old ones', () => {
    expect(f.species).toBeNull();
    expect(f.rank).toBeNull();
    expect(f.monsterName).toBeNull();
    expect(f.seed).toBe(7);
    expect(f.frames[0].monster.state).toBe('Windup');
  });
});
