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
