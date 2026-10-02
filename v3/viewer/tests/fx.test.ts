// Assumed module: src/fx.ts exports
//   hitstopTicks(events, t): number  (ticks of the hitstop event at tick t, else 0)
//   sparkSpec(event): null | {kind:'cone'|'radial'|'ring', count, color, spread, dirAngle}
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight } from '../src/data';
import { hitstopTicks, sparkSpec } from '../src/fx';

const fight = () => parseFight(JSON.parse(readFileSync(resolve(__dirname, 'fixtures/fight_small.json'), 'utf8')));
const hit = (impact: number, dx = 1, dy = 0) => ({ t: 8, kind: 'hit', victim: 1, part: 'Torso', x: 0, y: 0, dx, dy, impact, zone: 1, dmg: 10 }) as any;

describe('hitstopTicks', () => {
  it('returns 4 at t=8 and 0 elsewhere (fixture)', () => {
    const ev = fight().events;
    expect(hitstopTicks(ev, 8)).toBe(4);
    for (const t of [0, 2, 4, 6, 10, 12, 20]) expect(hitstopTicks(ev, t)).toBe(0);
  });
  it('empty events give 0', () => {
    expect(hitstopTicks([], 8)).toBe(0);
  });
});

describe('sparkSpec', () => {
  it('hit is a cone along (dx,dy) with spread <= PI/2', () => {
    const s: any = sparkSpec(hit(100, 0.6, 0.8));
    expect(s.kind).toBe('cone');
    expect(s.dirAngle).toBeCloseTo(Math.atan2(0.8, 0.6), 6);
    expect(s.spread).toBeLessThanOrEqual(Math.PI / 2);
    expect(s.spread).toBeGreaterThan(0);
  });
  it('hit direction angle follows a leftward hit', () => {
    const s: any = sparkSpec(hit(100, -1, 0));
    expect(Math.abs(s.dirAngle)).toBeCloseTo(Math.PI, 6);
  });
  it('spark count grows with impact', () => {
    const lo: any = sparkSpec(hit(20));
    const hi: any = sparkSpec(hit(200));
    expect(hi.count).toBeGreaterThan(lo.count);
    expect(lo.count).toBeGreaterThan(0);
  });
  it('clash is a radial burst over 2*PI in white-gold', () => {
    const s: any = sparkSpec({ t: 4, kind: 'clash', x: 700, y: 310, dx: 1, dy: 0, type: 'clash' } as any);
    expect(s.kind).toBe('radial');
    expect(s.spread).toBeCloseTo(2 * Math.PI, 6);
    const c = s.color as number;
    const r = (c >> 16) & 255, g = (c >> 8) & 255, b = c & 255;
    expect(r).toBeGreaterThanOrEqual(0xf0);
    expect(g).toBeGreaterThanOrEqual(0xc0);
    expect(b).toBeGreaterThanOrEqual(0x60);
  });
  it('block is a ring', () => {
    const s: any = sparkSpec({ t: 6, kind: 'block', x: 720, y: 280, impact: 60 } as any);
    expect(s.kind).toBe('ring');
  });
  it('dodge has no sparks', () => {
    expect(sparkSpec({ t: 12, kind: 'dodge', who: 0 } as any)).toBeNull();
  });
});
