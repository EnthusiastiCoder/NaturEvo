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
