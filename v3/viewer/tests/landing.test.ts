// Assumed module: src/landing.ts exports landingViewModel({runs, error?}) and renderLandingHtml(vm).
// Run shape: {name, seed, date, generations, fightGens: number[]}.
// vm: {empty: boolean, rows: [{name, openHref, ...}]}
import { describe, it, expect } from 'vitest';
import { landingViewModel, renderLandingHtml } from '../src/landing';
