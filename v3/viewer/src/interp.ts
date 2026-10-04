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
