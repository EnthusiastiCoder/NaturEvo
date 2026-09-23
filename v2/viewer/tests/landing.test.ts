// Assumed module: src/landing.ts (see v2/tests-index-viewer-landing.md)
import { describe, it, expect } from 'vitest';
import { landingViewModel, renderLandingHtml, EMPTY_COMMANDS } from '../src/landing';

const run = (name: string, extra = {}) => ({
  name, seed: '7', date: '2026-10-01', generations: 30,
  permadeath: { on: { outcome: 'Died', wins: 1, losses: 1, encounters: 2, perZone: {} }, off: undefined },
  keyResult: 'Died', files: [], ...extra,
}) as any;
