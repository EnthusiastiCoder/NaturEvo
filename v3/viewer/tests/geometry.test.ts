import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight, DEFAULT_GEOMETRY } from '../src/data';
import { wallsAt, bleedAt, suddenDeathActive } from '../src/walls';
import { shieldPlate } from '../src/robot-draw';
