// Assumed module: src/camera.ts exports cameraFor(frame, w, h) -> {scale, tx, ty}
// World coordinates are cm, y up; screen = (x*scale + tx, ty - y*scale).
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight } from '../src/data';
import { cameraFor } from '../src/camera';

const fight = () => parseFight(JSON.parse(readFileSync(resolve(__dirname, 'fixtures/fight_small.json'), 'utf8')));
const MARGIN = 20;

function extents(frame: any) {
  const xs: number[] = [];
  const ys: number[] = [];
  for (const g of frame.f) for (const p of g.parts) { xs.push(p[0], p[2]); ys.push(p[1], p[3]); }
  return { minX: Math.min(...xs), maxX: Math.max(...xs), minY: Math.min(...ys), maxY: Math.max(...ys) };
}
const toScreen = (c: any, x: number, y: number) => [x * c.scale + c.tx, c.ty - y * c.scale];
