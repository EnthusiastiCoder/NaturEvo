// Assumed module: src/interp.ts exporting sampleFight(frames, tSeconds, tickHz) and lerpAngle(a, b, t).
// Frame shape per FightLog: { pos:[x,y], facing:number, hp:number, stamina:number, action:string, state:string } per fighter.
import { describe, it, expect } from 'vitest';
import { sampleFight, lerpAngle } from '../src/interp';
