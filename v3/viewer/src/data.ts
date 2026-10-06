// v3 fight log parser (plan section 6). World units are cm, y up.
export interface Obstacle { x: number; w: number; h: number }
export interface Arena { width: number; height: number; obstacles: Obstacle[]; biome: string }
export type Seg = [number, number, number, number];
export interface FighterFrame {
  fx: number; facing: number; parts: Seg[]; sword: Seg; shield: Seg;
  hp: number[]; energy: number; posture: number; state: 'idle' | 'staggered' | 'fallen' | 'ko';
}
export interface Frame { t: number; f: [FighterFrame, FighterFrame] }
export type FightEvent = { t: number; kind: string; [k: string]: any };
export interface Tally { parry: number; block: number; dodge: number; hit: number }
export interface Fight {
  version: 3; arena: Arena; parts: string[]; dt: number; frames: Frame[]; events: FightEvent[];
  result: { outcome: string; ticks: number; defence: [Tally, Tally] };
}

const fail = (m: string): never => { throw new Error(`bad fight log: ${m}`); };
