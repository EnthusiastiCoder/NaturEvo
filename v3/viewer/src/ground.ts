import { Graphics } from 'pixi.js';
import type { Arena } from './data';

const BIOMES: Record<string, { sky: [number, number]; ground: number; line: number; wall: number; obst: number }> = {
  scrapland: { sky: [0x1b222b, 0x3a3228], ground: 0x4a3d33, line: 0x6b5a4a, wall: 0x2b3138, obst: 0x7a5236 },
  jungle: { sky: [0x10241a, 0x2c5a3a], ground: 0x2f4a26, line: 0x4a6b36, wall: 0x1d3322, obst: 0x5a4a30 },
};

/** Vector arena: sky gradient bands, ground slab, integer-aligned lines, obstacles, end walls. y world cm, scale px/cm. */
export function drawGround(g: Graphics, arena: Arena, scale: number, viewW: number, viewH: number, tx: number, ty: number, walls?: { lo: number; hi: number }, tick = 0): void {
  const b = BIOMES[arena.biome] ?? BIOMES.scrapland;
  g.clear();
  const bands = 12;
  for (let i = 0; i < bands; i++) {
    const k = i / (bands - 1);
    const mix = (a: number, c: number) => Math.round(a + (c - a) * k);
    const col = (mix((b.sky[0] >> 16) & 255, (b.sky[1] >> 16) & 255) << 16) | (mix((b.sky[0] >> 8) & 255, (b.sky[1] >> 8) & 255) << 8) | mix(b.sky[0] & 255, b.sky[1] & 255);
    g.rect(0, Math.floor((viewH * i) / bands), viewW, Math.ceil(viewH / bands) + 1).fill(col);
  }
  const gy = Math.round(ty);
  g.rect(0, gy, viewW, viewH - gy + 2).fill(b.ground);
  g.rect(0, gy, viewW, 3).fill(b.line);
  for (let y = gy + 14; y < viewH; y += 14) g.moveTo(0, y + 0.5).lineTo(viewW, y + 0.5).stroke({ width: 1, color: b.line, alpha: 0.25 });
  const wallW = 40 * scale;
  const x0 = Math.round(tx), x1 = Math.round(arena.width * scale + tx);
  g.rect(x0 - wallW, 0, wallW, gy).fill({ color: b.wall, alpha: 0.9 });
  g.rect(x1, 0, wallW, gy).fill({ color: b.wall, alpha: 0.9 });
  if (walls) { // sudden-death walls: solid slabs from each arena edge to the live wall position (what the sim collides with)
    const lx = Math.round(walls.lo * scale + tx), hx = Math.round(walls.hi * scale + tx);
    const closed = walls.lo > 50.5, pulse = closed ? 0.55 + 0.35 * Math.sin(tick / 6) : 0.35;
    const slab = (xa: number, xb: number, edge: number) => {
      g.rect(xa, 0, xb - xa, gy).fill({ color: 0x3a1a1a, alpha: closed ? 0.92 : 0.6 });
      for (let y = 0; y < gy; y += 28) g.poly([xa, y + 28, xb, y, xb, y + 14, xa, y + 42]).fill({ color: 0xff3b30, alpha: closed ? 0.18 : 0.08 });
      g.rect(edge - 2, 0, 4, gy).fill({ color: 0xff3b30, alpha: pulse });
    };
    slab(x0 - wallW, lx, lx);
    slab(hx, x1 + wallW, hx);
  }
  for (const o of arena.obstacles) {
    const ox = Math.round(o.x * scale + tx), ow = Math.round(o.w * scale), oh = Math.round(o.h * scale);
    g.rect(ox, gy - oh, ow, oh).fill(b.obst).stroke({ width: 1.5, color: 0x000000, alpha: 0.5 });
  }
}
