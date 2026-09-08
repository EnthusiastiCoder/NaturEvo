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
