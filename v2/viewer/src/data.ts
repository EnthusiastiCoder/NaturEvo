import { type Frame, type FighterFrame } from './interp';

export interface ArenaInfo { w: number; h: number; blocked: string[] | null }
export interface Fight { tickHz: number; frames: Frame[]; arena: ArenaInfo; seed: number; monsterIndex: number }
export interface MapData { w: number; h: number; tiles: string[]; zones: string[]; anchors: [number, number][] }
export interface Encounter { kind: string; zone: number; tile: [number, number]; monsterIndex: number; result: string; fight: Fight | null; fightUrl?: string }
export interface Run { map: MapData; path: [number, number][]; encounters: Encounter[]; outcome: string }

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type J = any;
const fighter = (f: J, i: number): FighterFrame => ({
  pos: [f.pos[i][0], f.pos[i][1]], facing: f.facing[i], hp: f.hp[i], stamina: f.stamina[i],
  action: f.action[i], state: f.state[i],
});

/** Convert the sim's FightLog JSON (per-fighter arrays, index 0 hero, 1 monster) into viewer frames. */
export function parseFight(j: J): Fight {
  const frames: Frame[] = j.frames.map((f: J) => ({ hero: fighter(f, 0), monster: fighter(f, 1) }));
  const a = j.arena;
  return {
    tickHz: j.tick_hz ?? 30, frames,
    arena: { w: a?.w ?? 48, h: a?.h ?? 32, blocked: a?.blocked ?? null },
    seed: j.seed ?? 1, monsterIndex: j.monster_index ?? 0,
  };
}

export function parseRun(j: J): Run {
  return {
    map: j.map, path: j.path, outcome: j.outcome ?? '',
    encounters: (j.encounters ?? []).map((e: J): Encounter => ({
      kind: String(e.kind ?? 'grass').toLowerCase(), zone: e.zone ?? 0, tile: e.tile, monsterIndex: e.monster_index ?? 0,
      result: String(e.result ?? ''), fight: e.fight && typeof e.fight === 'object' ? parseFight(e.fight) : null,
      fightUrl: typeof e.fight === 'string' ? e.fight : undefined,
    })),
  };
}

/** Deterministic demo data so the viewer works before any run exists. */
export function demoFight(seed: number): Fight {
  const frames: Frame[] = [];
  let hh = 100, mh = 100;
  for (let t = 0; t < 600; t++) {
    const ph = t % 90;
    const gap = ph < 40 ? 9 - ph * 0.16 : 2.6 + Math.sin(t * 0.05) * 0.4;
    const swing = ph >= 50 && ph < 56 ? 'Windup' : ph >= 56 && ph < 60 ? 'Active' : ph >= 60 && ph < 66 ? 'Recovery' : 'Idle';
    const mswing = ph >= 70 && ph < 76 ? 'Windup' : ph >= 76 && ph < 80 ? 'Active' : 'Idle';
    if (ph === 58) mh = Math.max(0, mh - 9);
    if (ph === 78 && (Math.floor(t / 90) % 2)) hh = Math.max(0, hh - 7);
    const orbit = Math.sin(t * 0.03) * 3;
    const hx = 24 - gap / 2, mx = 24 + gap / 2;
    frames.push({
      hero: { pos: [hx, 16 + orbit], facing: Math.atan2(-orbit * 0.1, 1), hp: hh, stamina: 100 - (t % 90) * 0.6, action: 'none', state: swing },
      monster: { pos: [mx, 16 - orbit * 0.5], facing: Math.PI, hp: mh, stamina: 100 - (t % 70) * 0.5, action: 'none',
        state: ph >= 76 && ph < 84 ? 'Blocking' : mswing },
    });
  }
  return { tickHz: 30, frames, arena: { w: 48, h: 32, blocked: null }, seed, monsterIndex: seed % 3 };
}
