import { Graphics } from 'pixi.js';
import type { Arena } from './data';

const BIOMES: Record<string, { sky: [number, number]; ground: number; line: number; wall: number; obst: number }> = {
  scrapland: { sky: [0x1b222b, 0x3a3228], ground: 0x4a3d33, line: 0x6b5a4a, wall: 0x2b3138, obst: 0x7a5236 },
  jungle: { sky: [0x10241a, 0x2c5a3a], ground: 0x2f4a26, line: 0x4a6b36, wall: 0x1d3322, obst: 0x5a4a30 },
};
