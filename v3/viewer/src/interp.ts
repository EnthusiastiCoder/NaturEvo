// Smooth sampling of a fight log at an arbitrary time (display rate independent).
export interface FighterFrame {
  pos: [number, number];
  facing: number;
  hp: number;
  stamina: number;
  action: Record<string, number | boolean> | string;
  state: string;
  atk: number;
  fx: number[];
  seen: boolean;
}
export interface Frame { hero: FighterFrame; monster: FighterFrame; ev: { t: string; [k: string]: unknown }[] }

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
    atk: a.atk, fx: a.fx, seen: a.seen,
  };
}
