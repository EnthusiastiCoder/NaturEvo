// Assumed module: src/fx.ts exports
//   hitstopTicks(events, t): number  (ticks of the hitstop event at tick t, else 0)
//   sparkSpec(event): null | {kind:'cone'|'radial'|'ring', count, color, spread, dirAngle}
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight } from '../src/data';
import { hitstopTicks, sparkSpec } from '../src/fx';
