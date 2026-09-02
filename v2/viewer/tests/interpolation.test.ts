// Assumed module: src/interp.ts exporting sampleFight(frames, tSeconds, tickHz) and lerpAngle(a, b, t).
// Frame shape per FightLog: { pos:[x,y], facing:number, hp:number, stamina:number, action:string, state:string } per fighter.
import { describe, it, expect } from 'vitest';
import { sampleFight, lerpAngle } from '../src/interp';

const f = (x: number, facing: number, hp: number, state = 'Idle') => ({
  hero: { pos: [x, 0], facing, hp, stamina: 100, action: 'none', state },
  monster: { pos: [10, 0], facing: 0, hp: 100, stamina: 100, action: 'none', state: 'Idle' },
});
const frames = [f(0, 0, 100), f(3, 1, 90), f(6, 2, 80, 'Windup')];
