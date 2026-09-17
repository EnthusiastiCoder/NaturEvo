// WS5 pure fx module (src/fx.ts). See v2/tests-index-ws5.md for the assumed API shapes.
import { describe, it, expect } from 'vitest';
import {
  STATUSES, statusTimer, tintStatus, knockbackOffset, KNOCK_TICKS, staggerShake, stunStarCount,
  dotEvents, statusStats, attackName, eyeAlpha,
} from '../src/fx';

const KEYS = ['knock', 'stagger', 'stun', 'confused', 'slowed', 'bleed', 'poison'];
const fxOf = (o: Record<string, number>) => KEYS.map((k) => o[k] ?? 0);

describe('status table', () => {
  it('covers the 7 statuses in sim order with unique colours, labels and info text', () => {
    expect(STATUSES.map((s) => s.key)).toEqual(KEYS);
    expect(new Set(STATUSES.map((s) => s.color)).size).toBe(7);
    expect(new Set(STATUSES.map((s) => s.label)).size).toBe(7);
    for (const s of STATUSES) expect(s.info.length).toBeGreaterThan(5);
  });
});

describe('statusTimer', () => {
  it('formats seconds with one decimal', () => {
    expect(statusTimer('stun', 24, 30)).toBe('stun 0.8');
    expect(statusTimer('bleed', 60, 30)).toBe('bleed 2.0');
    expect(statusTimer('poison', 15, 30)).toBe('poison 0.5');
  });
});

describe('tintStatus priority', () => {
  it('returns null with no status or knock only', () => {
    expect(tintStatus(fxOf({}))).toBeNull();
    expect(tintStatus(fxOf({ knock: 4 }))).toBeNull();
  });
  it('stun > stagger > confused > slowed > poison > bleed', () => {
    const order = ['stun', 'stagger', 'confused', 'slowed', 'poison', 'bleed'];
    for (let i = 0; i < order.length; i++) {
      const all = Object.fromEntries(order.slice(i).map((k) => [k, 5]));
      expect(tintStatus(fxOf(all))).toBe(order[i]);
    }
  });
  it('ignores knock when another status is active', () => {
    expect(tintStatus(fxOf({ knock: 3, bleed: 9 }))).toBe('bleed');
  });
});

describe('knockbackOffset', () => {
  it('KNOCK_TICKS is 6', () => expect(KNOCK_TICKS).toBe(6));
  it('visual lag trails the sim jump: full at start, zero at end, against the push direction', () => {
    const start = knockbackOffset(6, 6, 2, [1, 0]);
    expect(start[0]).toBeCloseTo(-2, 6);
    expect(start[1]).toBeCloseTo(0, 6);
    const end = knockbackOffset(0, 6, 2, [1, 0]);
    expect(end[0]).toBeCloseTo(0, 6);
    expect(end[1]).toBeCloseTo(0, 6);
  });
  it('shrinks monotonically and follows the direction', () => {
    let prev = Infinity;
    for (let r = 6; r >= 0; r--) {
      const m = Math.hypot(...knockbackOffset(r, 6, 3, [0, 1]));
      expect(m).toBeLessThanOrEqual(prev + 1e-9);
      prev = m;
    }
    const o = knockbackOffset(3, 6, 3, [0, 1]);
    expect(o[0]).toBeCloseTo(0, 6);
    expect(o[1]).toBeLessThan(0);
  });
  it('returns zero when total is 0', () => {
    expect(knockbackOffset(0, 0, 2, [1, 0])).toEqual([0, 0]);
  });
});

describe('staggerShake and stun stars', () => {
  it('shake is zero when not staggered, bounded and deterministic otherwise', () => {
    expect(staggerShake(1.3, 0)).toBe(0);
    const a = staggerShake(0.37, 10);
    expect(a).toBe(staggerShake(0.37, 10));
    for (let i = 0; i < 50; i++) expect(Math.abs(staggerShake(i * 0.013, 8))).toBeLessThanOrEqual(1);
    expect(Array.from({ length: 50 }, (_, i) => staggerShake(i * 0.013, 8)).some((v) => Math.abs(v) > 0.1)).toBe(true);
  });
  it('star count is 0 without stun and 1..5 with it', () => {
    expect(stunStarCount(0)).toBe(0);
    for (const r of [1, 10, 20]) {
      expect(stunStarCount(r)).toBeGreaterThanOrEqual(1);
      expect(stunStarCount(r)).toBeLessThanOrEqual(5);
    }
  });
});

describe('dotEvents', () => {
  it('keeps only dot events, in order', () => {
    const ev = [
      { t: 'hit', by: 0, atk: 0, dmg: 5, blocked: false, punish: false, fx: [] },
      { t: 'dot', to: 1, kind: 'poison', dmg: 1 },
      { t: 'parry', by: 0 },
      { t: 'dot', to: 0, kind: 'bleed', dmg: 2 },
    ];
    expect(dotEvents(ev as never)).toEqual([ev[1], ev[3]]);
    expect(dotEvents([])).toEqual([]);
  });
});

const hit = (by: number, fx: number[], punish = false) => ({ t: 'hit', by, atk: 0, dmg: 5, blocked: false, punish, fx });
const frame = (ev: unknown[]) => ({ hero: {}, monster: {}, ev });
describe('statusStats', () => {
  const frames = [
    frame([]),
    frame([hit(0, [2], true), { t: 'parry', by: 1 }]),                  // hero applies stun, punishes; monster parries
    frame([hit(1, [5, 6]), hit(0, [2])]),                               // monster applies bleed+poison; hero stun again
    frame([{ t: 'parry', by: 0 }, hit(0, [], true)]),                   // after upTo for some queries
  ];
  it('counts statuses applied BY each side, punishes and parries', () => {
    const s = statusStats(frames as never, 2);
    expect(s.hero.statuses.stun).toBe(2);
    expect(s.hero.punishes).toBe(1);
    expect(s.hero.parries).toBe(0);
    expect(s.monster.statuses.bleed).toBe(1);
    expect(s.monster.statuses.poison).toBe(1);
    expect(s.monster.statuses.stun).toBe(0);
    expect(s.monster.parries).toBe(1);
    expect(s.monster.punishes).toBe(0);
  });
  it('includes frame upTo and respects it', () => {
    const all = statusStats(frames as never, 3);
    expect(all.hero.parries).toBe(1);
    expect(all.hero.punishes).toBe(2);
    const none = statusStats(frames as never, 0);
    expect(none.hero.punishes).toBe(0);
    expect(Object.values(none.hero.statuses).every((v) => v === 0)).toBe(true);
    expect(Object.keys(none.hero.statuses)).toEqual(KEYS);
  });
  it('tolerates frames without ev', () => {
    expect(statusStats([{ hero: {}, monster: {} }] as never, 0).hero.punishes).toBe(0);
  });
});
