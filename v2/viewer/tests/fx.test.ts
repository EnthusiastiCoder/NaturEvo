// WS5 pure fx module (src/fx.ts). See v2/tests-index-ws5.md for the assumed API shapes.
import { describe, it, expect } from 'vitest';
import {
  STATUSES, statusTimer, tintStatus, knockbackOffset, KNOCK_TICKS, staggerShake, stunStarCount,
  dotEvents, statusStats, attackName, eyeAlpha,
} from '../src/fx';

const KEYS = ['knock', 'stagger', 'stun', 'confused', 'slowed', 'bleed', 'poison'];
const fxOf = (o: Record<string, number>) => KEYS.map((k) => o[k] ?? 0);
