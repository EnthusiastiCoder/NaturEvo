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
