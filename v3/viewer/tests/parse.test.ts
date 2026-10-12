// Assumed module: src/data.ts exports parseFight(json: unknown): Fight (v3 log, plan section 6)
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight } from '../src/data';

const raw = () => JSON.parse(readFileSync(resolve(__dirname, 'fixtures/fight_small.json'), 'utf8'));

describe('parseFight', () => {
  it('parses arena, parts and dt', () => {
    const f = parseFight(raw());
    expect(f.version).toBe(3);
    expect(f.arena.width).toBe(1400);
    expect(f.arena.height).toBe(600);
    expect(f.arena.biome).toBe('scrapland');
    expect(f.arena.obstacles).toEqual([{ x: 700, w: 80, h: 90 }]);
    expect(f.parts).toHaveLength(13);
    expect(f.parts[0]).toBe('Head');
    expect(f.dt).toBeCloseTo(0.0333, 4);
  });
  it('parses 12 frames with two fighters of 13 parts each', () => {
    const f = parseFight(raw());
    expect(f.frames).toHaveLength(12);
    expect(f.frames[0].t).toBe(0);
    expect(f.frames[11].t).toBe(22);
    for (const fr of f.frames) {
      expect(fr.f).toHaveLength(2);
      for (const g of fr.f) {
        expect(g.parts).toHaveLength(13);
        expect(g.hp).toHaveLength(13);
        expect(g.sword).toHaveLength(4);
        expect(g.shield).toHaveLength(4);
        expect(['idle', 'staggered', 'fallen', 'ko']).toContain(g.state);
      }
    }
  });
  it('parses all event kinds with their fields', () => {
    const f = parseFight(raw());
    const kinds = f.events.map((e: any) => e.kind);
    expect(kinds).toEqual(['clash', 'block', 'hit', 'hitstop', 'stagger', 'dodge', 'destroyed', 'fall', 'clash', 'ko']);
    const hit: any = f.events.find((e: any) => e.kind === 'hit');
    expect(hit).toMatchObject({ t: 8, victim: 1, part: 'Torso', x: 850, y: 280, impact: 120, zone: 1, dmg: 66 });
    expect(hit.dx).toBeCloseTo(0.97, 5);
    expect(hit.dy).toBeCloseTo(-0.24, 5);
    const d: any = f.events.find((e: any) => e.kind === 'destroyed');
    expect(d).toMatchObject({ who: 1, part: 'FootR' });
  });
  it('parses the result with per-fighter defence tallies', () => {
    const f = parseFight(raw());
    expect(f.result.outcome).toBe('win_a');
    expect(f.result.ticks).toBe(24);
    expect(f.result.defence).toEqual([
      { parry: 1, block: 1, dodge: 1, hit: 1 },
      { parry: 0, block: 0, dodge: 0, hit: 2 },
    ]);
  });
  it('throws on a version other than 3', () => {
    const j = raw();
    j.version = 2;
    expect(() => parseFight(j)).toThrow(/version/i);
  });
  it('normalises the real sim log (arena and events in metres) to centimetres', () => {
    const j = raw();
    j.arena = { width: 14, height: 6, obstacles: [{ x: 2.8, w: 0.99, h: 1.16 }], biome: 'jungle' };
    j.events = [{ t: 0, kind: 'clash', x: 6.93, y: 4.26, dx: 0.5, dy: 0.8, type: 'parry' }];
    const f = parseFight(j);
    expect(f.arena.width).toBe(1400);
    expect(f.arena.height).toBe(600);
    expect(f.arena.obstacles[0]).toEqual({ x: 280, w: 99, h: 116 });
    expect(f.events[0].x).toBeCloseTo(693);
    expect(f.events[0].y).toBeCloseTo(426);
    expect(f.events[0].dx).toBe(0.5);
  });
});
