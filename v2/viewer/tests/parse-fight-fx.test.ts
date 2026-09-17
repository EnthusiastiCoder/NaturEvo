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

describe('parseFight new logs', () => {
  const ev = [{ t: 'hit', by: 0, atk: 1, dmg: 8, blocked: false, punish: true, fx: [2] }, { t: 'parry', by: 1 }];
  const f = parseFight(base(
    { species: ['hero', 'warrior'], rank: ['normal', 'elite'], monster_name: 'Grub' },
    { atk: [1, -1], fx: [[0, 0, 5, 0, 0, 0, 0], [0, 0, 0, 0, 0, 3, 9]], seen: [true, false], ev },
  ));
  it('reads atk, fx, seen per fighter (index 0 hero, 1 monster)', () => {
    expect(f.frames[0].hero.atk).toBe(1);
    expect(f.frames[0].monster.atk).toBe(-1);
    expect(f.frames[0].hero.fx).toEqual([0, 0, 5, 0, 0, 0, 0]);
    expect(f.frames[0].monster.fx).toEqual([0, 0, 0, 0, 0, 3, 9]);
    expect(f.frames[0].hero.seen).toBe(true);
    expect(f.frames[0].monster.seen).toBe(false);
  });
  it('reads events unchanged', () => {
    expect(f.frames[0].ev).toEqual(ev);
  });
  it('reads species, rank and monster name', () => {
    expect(f.species).toEqual(['hero', 'warrior']);
    expect(f.rank).toEqual(['normal', 'elite']);
    expect(f.monsterName).toBe('Grub');
  });
});
