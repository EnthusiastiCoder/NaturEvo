// Smooth sampling of a fight log at an arbitrary time (display rate independent).
export interface FighterFrame {
  pos: [number, number];
  facing: number;
  hp: number;
  stamina: number;
  action: Record<string, number | boolean> | string;
  state: string;
}
export interface Frame { hero: FighterFrame; monster: FighterFrame }

export function lerpAngle(a: number, b: number, t: number): number {
  const d = ((((b - a + Math.PI) % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI)) - Math.PI;
  return a + d * t;
}
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

function mix(a: FighterFrame, b: FighterFrame, t: number): FighterFrame {
  return {
    pos: [lerp(a.pos[0], b.pos[0], t), lerp(a.pos[1], b.pos[1], t)],
    facing: lerpAngle(a.facing, b.facing, t),
    hp: lerp(a.hp, b.hp, t),
    stamina: lerp(a.stamina, b.stamina, t),
    action: a.action, // discrete values snap to the earlier tick
    state: a.state,
  };
}

export function sampleFight(frames: Frame[], tSeconds: number, tickHz: number): Frame {
  const last = frames.length - 1;
  const s = Math.round(Math.max(0, tSeconds) * tickHz * 1e9) / 1e9;
  if (s >= last) return frames[last];
  const i = Math.floor(s);
  const f = s - i;
  if (f === 0) return frames[i];
  const a = frames[i], b = frames[i + 1];
  return { hero: mix(a.hero, b.hero, f), monster: mix(a.monster, b.monster, f) };
}

export interface SideStats { hits: number; blocks: number; dodges: number }
/** Events up to and including frame `upTo`: hits landed (opponent hp drops), entries into Blocking and Dodging. */
export function fightStats(frames: Frame[], upTo: number): { hero: SideStats; monster: SideStats } {
  const out = { hero: { hits: 0, blocks: 0, dodges: 0 }, monster: { hits: 0, blocks: 0, dodges: 0 } };
  const end = Math.min(frames.length - 1, Math.max(0, upTo));
  // the sim's state flickers frame to frame: an action counts once per 8-frame burst
  const last = { hero: { Blocking: -99, Dodging: -99 }, monster: { Blocking: -99, Dodging: -99 } };
  for (let i = 1; i <= end; i++) {
    const a = frames[i - 1], b = frames[i];
    if (a.monster.hp - b.monster.hp > 0.5) out.hero.hits++;
    if (a.hero.hp - b.hero.hp > 0.5) out.monster.hits++;
    for (const k of ['hero', 'monster'] as const) {
      for (const [st, key] of [['Blocking', 'blocks'], ['Dodging', 'dodges']] as const) {
        if (b[k].state !== st) continue;
        if (i - last[k][st] > 8) out[k][key]++;
        last[k][st] = i;
      }
    }
  }
  return out;
}
export const statsLine = (name: string, s: SideStats) => `${name}: ${s.hits} hits, ${s.blocks} blocks, ${s.dodges} dodges`;
