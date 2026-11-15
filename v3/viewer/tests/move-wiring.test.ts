import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight } from '../src/data';
import { sampleFrame } from '../src/scene';
import { hudMoveLine, moveCallout } from '../src/move-fx';

const load = (n: string) => parseFight(JSON.parse(readFileSync(resolve(__dirname, 'fixtures', n), 'utf8')));
describe('move fx wiring', () => {
  it('interpolated frames keep mv so the HUD line and callout are scrub-deterministic', () => {
    const f = load('fight_moves.json');
    const fr = sampleFrame(f, 5);
    expect(hudMoveLine(fr.f[0])).toBe('Overhead · Windup');
    expect(moveCallout(f.events, fr, 5, f.dt)).toEqual(moveCallout(f.events, sampleFrame(f, 5), 5, f.dt));
  });
  it('old logs sample without mv', () => {
    const f = load('fight_small.json');
    expect(sampleFrame(f, 1).f[0].mv).toBeUndefined();
  });
});
