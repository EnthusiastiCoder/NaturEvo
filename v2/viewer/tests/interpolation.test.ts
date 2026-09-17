// Assumed module: src/interp.ts exporting sampleFight(frames, tSeconds, tickHz) and lerpAngle(a, b, t).
// Frame shape per FightLog: { pos:[x,y], facing:number, hp:number, stamina:number, action:string, state:string } per fighter.
import { describe, it, expect } from 'vitest';
import { sampleFight, lerpAngle } from '../src/interp';

const f = (x: number, facing: number, hp: number, state = 'Idle') => ({
  hero: { pos: [x, 0], facing, hp, stamina: 100, action: 'none', state },
  monster: { pos: [10, 0], facing: 0, hp: 100, stamina: 100, action: 'none', state: 'Idle' },
});
const frames = [f(0, 0, 100), f(3, 1, 90), f(6, 2, 80, 'Windup')];

describe('fight interpolation', () => {
  it('returns frame 0 at t=0', () => {
    expect(sampleFight(frames, 0, 30).hero.pos[0]).toBe(0);
  });
  it('lerps position halfway between ticks (tick 1/30 s)', () => {
    const s = sampleFight(frames, 0.5 / 30, 30);
    expect(s.hero.pos[0]).toBeCloseTo(1.5, 6);
    expect(s.hero.hp).toBeCloseTo(95, 6);
  });
  it('is exact on a tick boundary', () => {
    expect(sampleFight(frames, 1 / 30, 30).hero.pos[0]).toBeCloseTo(3, 6);
  });
  it('clamps beyond the end to the last frame', () => {
    expect(sampleFight(frames, 99, 30).hero.pos[0]).toBe(6);
  });
  it('clamps negative time to frame 0', () => {
    expect(sampleFight(frames, -1, 30).hero.pos[0]).toBe(0);
  });
  it('discrete state snaps to the earlier frame, not blended', () => {
    expect(sampleFight(frames, 1.9 / 30, 30).hero.state).toBe('Idle');
    expect(sampleFight(frames, 2 / 30, 30).hero.state).toBe('Windup');
  });
  it('lerpAngle takes the short way round across +-PI', () => {
    const r = lerpAngle(Math.PI - 0.1, -Math.PI + 0.1, 0.5);
    expect(Math.abs(Math.abs(r) - Math.PI)).toBeLessThan(1e-6);
  });
  it('carries atk, fx, seen and ev from the earlier frame (not blended)', () => {
    const mk = (x: number, atk: number, fx: number[], seen: boolean, ev: unknown[]) => ({
      hero: { pos: [x, 0], facing: 0, hp: 100, stamina: 100, action: 'none', state: 'Idle', atk, fx, seen },
      monster: { pos: [10, 0], facing: 0, hp: 100, stamina: 100, action: 'none', state: 'Idle', atk: -1, fx: [0, 0, 0, 0, 0, 0, 0], seen: false },
      ev,
    });
    const e0 = [{ t: 'parry', by: 0 }];
    const fs = [mk(0, 2, [0, 0, 8, 0, 0, 0, 0], true, e0), mk(4, 0, [0, 0, 7, 0, 0, 0, 0], false, [])] as never;
    const s = sampleFight(fs, 0.5 / 30, 30) as never as { hero: { atk: number; fx: number[]; seen: boolean }; ev: unknown[] };
    expect(s.hero.atk).toBe(2);
    expect(s.hero.fx).toEqual([0, 0, 8, 0, 0, 0, 0]);
    expect(s.hero.seen).toBe(true);
    expect(s.ev).toEqual(e0);
    const at1 = sampleFight(fs, 1 / 30, 30) as never as { hero: { atk: number }; ev: unknown[] };
    expect(at1.hero.atk).toBe(0);
    expect(at1.ev).toEqual([]);
  });
  it('is independent of display refresh rate (same t, same sample)', () => {
    const a = sampleFight(frames, 0.017, 30);
    const b = sampleFight(frames, 0.017, 30);
    expect(a).toEqual(b);
  });
});
