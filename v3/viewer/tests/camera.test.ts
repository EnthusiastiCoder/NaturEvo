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

describe.each([[1282, 642], [1920, 1080]])('cameraFor at %ix%i', (w, h) => {
  it('keeps every part endpoint inside the viewport with a 20px margin', () => {
    const fr = fight().frames[0];
    const c = cameraFor(fr, w, h);
    const e = extents(fr);
    for (const [x, y] of [[e.minX, e.minY], [e.maxX, e.maxY], [e.minX, e.maxY], [e.maxX, e.minY]]) {
      const [sx, sy] = toScreen(c, x, y);
      expect(sx).toBeGreaterThanOrEqual(MARGIN);
      expect(sx).toBeLessThanOrEqual(w - MARGIN);
      expect(sy).toBeGreaterThanOrEqual(MARGIN);
      expect(sy).toBeLessThanOrEqual(h - MARGIN);
    }
  });
  it('robot height is at least 45% of canvas height', () => {
    const fr = fight().frames[0];
    const c = cameraFor(fr, w, h);
    const g = fr.f[0];
    const ys = g.parts.flatMap((p: number[]) => [p[1], p[3]]);
    const robotH = (Math.max(...ys) - Math.min(...ys)) * c.scale;
    expect(robotH).toBeGreaterThanOrEqual(0.45 * h);
  });
  it('fighters 1000 cm apart still fit with margin', () => {
    const fr = JSON.parse(JSON.stringify(fight().frames[0]));
    const dx = 1000 - (fr.f[1].fx - fr.f[0].fx);
    const shift = (g: any, d: number) => {
      g.fx += d;
      for (const p of g.parts) { p[0] += d; p[2] += d; }
    };
    shift(fr.f[0], -dx / 2);
    shift(fr.f[1], dx / 2);
    expect(fr.f[1].fx - fr.f[0].fx).toBe(1000);
    const c = cameraFor(fr, w, h);
    const e = extents(fr);
    const [x0] = toScreen(c, e.minX, 0);
    const [x1] = toScreen(c, e.maxX, 0);
    expect(x0).toBeGreaterThanOrEqual(MARGIN);
    expect(x1).toBeLessThanOrEqual(w - MARGIN);
  });
});
