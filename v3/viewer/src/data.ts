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
/** Collision geometry in cm, from the log's `geometry` block (shieldAlong is a 0..1 fraction of the forearm). */
export interface Geometry {
  headR: number; neckR: number; shieldLen: number; shieldAlong: number; shieldR: number;
  swordLen: number; swordR: number; radii: number[]; // radii in part order Head Torso Pelvis UArmL UArmR FArmL FArmR ThighL ThighR ShinL ShinR FootL FootR
}
/** Used for logs written before the geometry block existed. */
export const DEFAULT_GEOMETRY: Geometry = {
  headR: 23, neckR: 9, shieldLen: 70, shieldAlong: 0.62, shieldR: 5, swordLen: 220, swordR: 5,
  radii: [23, 30, 26, 11, 11, 10, 10, 15, 15, 12, 12, 9, 9],
};
/** Closing-walls mechanic (cm, ticks at 60/s). */
export interface SuddenDeath { startTick: number; speedCm: number; minWidth: number; bleedBase: number; bleedGrowth: number }
export interface Fight {
  version: 3; arena: Arena; parts: string[]; dt: number; frames: Frame[]; events: FightEvent[];
  geometry: Geometry; suddenDeath: SuddenDeath | null;
  result: { outcome: string; ticks: number; defence: [Tally, Tally] };
}

const num = (v: unknown): v is number => typeof v === 'number' && Number.isFinite(v);

function parseGeometry(g: any): Geometry {
  const ok = g && [g.head_r, g.neck_r, g.shield_len, g.shield_along, g.shield_r, g.sword_len, g.sword_r].every(num)
    && Array.isArray(g.radii) && g.radii.length === 13 && g.radii.every(num);
  if (!ok) return DEFAULT_GEOMETRY;
  return { headR: g.head_r, neckR: g.neck_r, shieldLen: g.shield_len, shieldAlong: g.shield_along > 1 ? g.shield_along / 100 : g.shield_along,
    shieldR: g.shield_r, swordLen: g.sword_len, swordR: g.sword_r, radii: [...g.radii] };
}
function parseSuddenDeath(s: any): SuddenDeath | null {
  if (!s || !num(s.start_tick) || !num(s.speed) || !num(s.min_width)) return null;
  return { startTick: s.start_tick, speedCm: s.speed * 100, minWidth: s.min_width * 100,
    bleedBase: num(s.bleed_base) ? s.bleed_base : 4, bleedGrowth: num(s.bleed_growth) ? s.bleed_growth : 0.25 };
}

const fail = (m: string): never => { throw new Error(`bad fight log: ${m}`); };

export function parseFight(json: unknown): Fight {
  const j = json as any;
  if (!j || j.version !== 3) fail(`unsupported version ${j?.version}, expected 3`);
  if (!j.arena || !Array.isArray(j.parts) || !Array.isArray(j.frames)) fail('missing arena, parts or frames');
  const a = j.arena;
  // The sim writes arena and event coordinates in metres while frame segments are cm; the fixture is all cm.
  const k = a.width < 100 ? 100 : 1;
  const c = (v: number) => Math.round(v * k * 100) / 100;
  const events: FightEvent[] = (Array.isArray(j.events) ? j.events : []).map((e: FightEvent) =>
    k === 1 || typeof e.x !== 'number' ? e : { ...e, x: c(e.x), y: c(e.y) });
  return {
    version: 3,
    arena: { width: c(a.width), height: c(a.height), obstacles: (a.obstacles ?? []).map((o: any) => ({ x: c(o.x), w: c(o.w), h: c(o.h) })), biome: a.biome ?? 'scrapland' },
    parts: j.parts,
    dt: j.dt,
    frames: j.frames,
    events,
    result: j.result ?? { outcome: 'draw', ticks: 0, defence: [{ parry: 0, block: 0, dodge: 0, hit: 0 }, { parry: 0, block: 0, dodge: 0, hit: 0 }] },
  };
}
