import { describe, it, expect } from 'vitest';
import { parseRun, resolveFightUrl, resolveFights, mergeDuels, type Fight } from '../src/data';

const fightJson = { tick_hz: 30, seed: 1, monster_index: 0, arena: { w: 48, h: 32 },
  frames: [{ pos: [[1, 1], [2, 2]], facing: [0, 0], hp: [100, 100], stamina: [100, 100], action: ['none', 'none'], state: ['Idle', 'Idle'] }] };
const runJson = (fight: unknown) => ({ map: { w: 1, h: 1, tiles: ['.'], zones: ['0'], anchors: [] }, path: [[0, 0]], encounters: [{ kind: 'grass', tile: [0, 0], fight }] });
