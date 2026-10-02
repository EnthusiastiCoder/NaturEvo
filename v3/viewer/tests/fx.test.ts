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
