import { type Frame, type FighterFrame } from './interp';

export interface ArenaInfo { w: number; h: number; blocked: string[] | null }
export interface Fight { tickHz: number; frames: Frame[]; arena: ArenaInfo; seed: number; monsterIndex: number; species: string[] | null; rank: string[] | null; monsterName: string | null }
export interface MapData { w: number; h: number; tiles: string[]; zones: string[]; anchors: [number, number][] }
export interface Encounter { kind: string; zone: number; tile: [number, number]; monsterIndex: number; result: string; fight: Fight | null; fightUrl?: string }
export interface Run { map: MapData; path: [number, number][]; encounters: Encounter[]; outcome: string }

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type J = any;
const fighter = (f: J, i: number): FighterFrame => ({
  pos: [f.pos[i][0], f.pos[i][1]], facing: f.facing[i], hp: f.hp[i], stamina: f.stamina[i],
  action: f.action[i], state: f.state[i],
  atk: f.atk?.[i] ?? -1, fx: f.fx?.[i] ? [...f.fx[i]] : [0, 0, 0, 0, 0, 0, 0], seen: f.seen?.[i] ?? false,
});

/** Convert the sim's FightLog JSON (per-fighter arrays, index 0 hero, 1 monster) into viewer frames. */
export function parseFight(j: J): Fight {
  const frames: Frame[] = j.frames.map((f: J) => ({ hero: fighter(f, 0), monster: fighter(f, 1), ev: f.ev ?? [] }));
  const a = j.arena;
  return {
    tickHz: j.tick_hz ?? 30, frames,
    arena: { w: a?.w ?? 48, h: a?.h ?? 32, blocked: a?.blocked ?? null },
    seed: j.seed ?? 1, monsterIndex: j.monster_index ?? 0,
    species: j.species ?? null, rank: j.rank ?? null, monsterName: j.monster_name ?? null,
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
    const c = Math.floor(t / 90) % 7;
    const mfx = [0, 0, 0, 0, 0, 0, 0], hfx = [0, 0, 0, 0, 0, 0, 0];
    if (ph >= 58) mfx[c] = Math.max(1, 32 - (ph - 58));
    if (ph >= 20 && ph < 50) hfx[(c + 3) % 7] = 50 - ph;
    const ev: Frame['ev'] = [];
    if (ph === 58) ev.push({ t: 'hit', by: 0, atk: 0, dmg: 9, blocked: false, punish: true, fx: [c] });
    if (ph === 78) ev.push({ t: 'hit', by: 1, atk: 1, dmg: 7, blocked: false, punish: false, fx: [(c + 3) % 7] });
    if (ph === 62) ev.push({ t: 'parry', by: 1 });
    if (ph % 10 === 0 && mfx[6] > 0) ev.push({ t: 'dot', to: 1, kind: 'poison', dmg: 1 });
    if (ph % 10 === 5 && mfx[5] > 0) ev.push({ t: 'dot', to: 1, kind: 'bleed', dmg: 1 });
    frames.push({
      hero: { pos: [hx, 16 + orbit], facing: Math.atan2(-orbit * 0.1, 1), hp: hh, stamina: 100 - (t % 90) * 0.6, action: 'none', state: swing,
        atk: swing === 'Idle' ? -1 : c % 3, fx: hfx, seen: ph < 70 },
      monster: { pos: [mx, 16 - orbit * 0.5], facing: Math.PI, hp: mh, stamina: 100 - (t % 70) * 0.5, action: 'none',
        state: ph >= 76 && ph < 84 ? 'Blocking' : mswing, atk: mswing === 'Idle' ? -1 : c % 2, fx: mfx, seen: true },
      ev,
    });
  }
  return { tickHz: 30, frames, arena: { w: 48, h: 32, blocked: null }, seed, monsterIndex: seed % 3, species: ['hero', 'minion'], rank: ['normal', 'normal'], monsterName: null };
}

export function demoRun(): Run {
  const n = 40;
  const tiles: string[] = [], zones: string[] = [];
  for (let y = 0; y < n; y++) {
    let tr = '', zr = '';
    for (let x = 0; x < n; x++) {
      const border = x === 0 || y === 0 || x === n - 1 || y === n - 1;
      const h = Math.sin(x * 12.9898 + y * 78.233) * 43758.5453; const r = h - Math.floor(h);
      tr += border ? 'T' : r < 0.12 ? 'T' : r < 0.16 ? 'R' : (x > 14 && x < 19 && y > 6 && y < 30) ? 'W' : r < 0.45 ? ',' : '.';
      zr += String(Math.min(3, Math.floor((x + y) / 20)));
    }
    tiles.push(tr); zones.push(zr);
  }
  const set = (x: number, y: number, c: string) => { tiles[y] = tiles[y].slice(0, x) + c + tiles[y].slice(x + 1); };
  const path: [number, number][] = [];
  for (let x = 3; x <= 36; x++) { path.push([x, 3]); set(x, 3, '.'); }
  for (let y = 4; y <= 36; y++) { path.push([36, y]); set(36, y, '.'); }
  set(3, 3, 'C'); set(36, 36, 'C');
  const encs: Encounter[] = [
    { kind: 'grass', zone: 0, tile: [10, 3], monsterIndex: 0, result: 'Won', fight: demoFight(1) },
    { kind: 'elite', zone: 1, tile: [30, 3], monsterIndex: 1, result: 'Won', fight: demoFight(2) },
    { kind: 'boss', zone: 3, tile: [36, 36], monsterIndex: 2, result: 'Won', fight: demoFight(3) },
  ];
  return { map: { w: n, h: n, tiles, zones, anchors: [[3, 3], [36, 36]] }, path, encounters: encs, outcome: 'Won' };
}

/** Fight file names in a run are relative to the run file, so resolve against its URL. */
export function resolveFightUrl(fight: string, runUrl: string): string {
  return new URL(fight, runUrl).href;
}

/** Load every referenced fight; returns one message per failure so the UI can show it. */
export async function resolveFights(run: Run, runUrl: string, fetchFn: (u: string) => Promise<Response> = (u) => fetch(u)): Promise<string[]> {
  const errors: string[] = [];
  await Promise.all(run.encounters.map(async (e) => {
    if (e.fight || !e.fightUrl) return;
    try {
      const r = await fetchFn(resolveFightUrl(e.fightUrl, runUrl));
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      e.fight = parseFight(await r.json());
    } catch (err) { errors.push(`${e.fightUrl}: ${err instanceof Error ? err.message : String(err)}`); }
  }));
  return errors;
}

export function mergeDuels<T>(fromRun: T[], requested: T | null): T[] {
  return requested ? [requested, ...fromRun] : fromRun;
}
