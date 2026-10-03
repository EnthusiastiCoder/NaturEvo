// Assumed module: src/landing.ts exports landingViewModel({runs, error?}) and renderLandingHtml(vm).
// Run shape: {name, seed, date, generations, fightGens: number[]}.
// vm: {empty: boolean, rows: [{name, openHref, ...}]}
import { describe, it, expect } from 'vitest';
import { landingViewModel, renderLandingHtml } from '../src/landing';

const run = (name: string, fightGens: number[] = [0, 10, 200]) =>
  ({ name, seed: '7', date: '2026-10-24', generations: 200, fightGens }) as any;
