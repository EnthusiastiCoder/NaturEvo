// Assumed modules:
//   src/hud.ts: partHealthColor(hp): string (hex, 4 tiers), hudModel(frame), defenceTallyModel(defence)
//   src/renderer.ts: rendererOptions(dpr)
//   src/robot-draw.ts must exist and be vector-only
import { describe, it, expect } from 'vitest';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { partHealthColor, hudModel, defenceTallyModel } from '../src/hud';
import { rendererOptions } from '../src/renderer';
import { parseFight } from '../src/data';
