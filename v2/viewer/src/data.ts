import { type Frame, type FighterFrame } from './interp';

export interface ArenaInfo { w: number; h: number; blocked: string[] | null }
export interface Fight { tickHz: number; frames: Frame[]; arena: ArenaInfo; seed: number; monsterIndex: number }
export interface MapData { w: number; h: number; tiles: string[]; zones: string[]; anchors: [number, number][] }
export interface Encounter { kind: string; zone: number; tile: [number, number]; monsterIndex: number; result: string; fight: Fight | null; fightUrl?: string }
export interface Run { map: MapData; path: [number, number][]; encounters: Encounter[]; outcome: string }
