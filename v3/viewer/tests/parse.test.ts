// Assumed module: src/data.ts exports parseFight(json: unknown): Fight (v3 log, plan section 6)
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight } from '../src/data';

const raw = () => JSON.parse(readFileSync(resolve(__dirname, 'fixtures/fight_small.json'), 'utf8'));
