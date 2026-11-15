import { describe, it, expect } from 'vitest';
import { recentEvents, ageParticles, spawnParticles, sparkSpec } from '../src/fx';

const ev = (t: number, kind = 'hit') => ({ t, kind, victim: 1, part: 'Torso', x: 5, y: 5, dx: 1, dy: 0, impact: 200, zone: 1, dmg: 10 }) as any;

describe('recentEvents (scrub replays sparks)', () => {
  it('returns events in [tick-window, tick] with age in seconds', () => {
    const r = recentEvents([ev(5), ev(20), ev(30)], 24, 12);
    expect(r.map((x) => x.e.t)).toEqual([20]);
    expect(r[0].age).toBeCloseTo(4 / 60, 5);
  });
  it('excludes future events', () => {
    expect(recentEvents([ev(30)], 24, 12)).toEqual([]);
  });
});

describe('ageParticles', () => {
  it('moves particles and drops expired ones', () => {
    const ps = spawnParticles(sparkSpec(ev(1))!, 5, 5, 1);
    const moved = ageParticles(ps, 0.05);
    expect(moved.length).toBeGreaterThan(0);
    expect(moved[0].x).not.toBe(5);
    expect(ageParticles(ps, 5)).toEqual([]);
  });
});
