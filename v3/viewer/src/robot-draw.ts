// Vector-only robot drawing: every part is a Graphics capsule. Endpoints come straight from the log.
import { Graphics } from 'pixi.js';
import type { FighterFrame, Seg } from './data';
import { partDamageStyle } from './damage-style';

// order: Head Torso Pelvis UArmL UArmR FArmL FArmR ThighL ThighR ShinL ShinR FootL FootR
const RADIUS_CM = [14, 30, 26, 11, 11, 10, 10, 15, 15, 12, 12, 9, 9];
const BACK = new Set([3, 5, 7, 9, 11]); // left limbs drawn darker (far side)
export const PART_Z = [8, 5, 4, 2, 6, 3, 7, 1, 2, 1, 2, 0, 1];

export function shade(hex: number, f: number): number {
  const r = Math.round(((hex >> 16) & 255) * f), g = Math.round(((hex >> 8) & 255) * f), b = Math.round((hex & 255) * f);
  return (Math.min(255, r) << 16) | (Math.min(255, g) << 8) | Math.min(255, b);
}
export function desat(hex: number, k: number): number {
  const r = (hex >> 16) & 255, g = (hex >> 8) & 255, b = hex & 255;
  const y = 0.3 * r + 0.59 * g + 0.11 * b;
  const m = (c: number) => Math.round(c + (y - c) * k);
  return (m(r) << 16) | (m(g) << 8) | m(b);
}

function capsule(g: Graphics, s: Seg, r: number, sc: number, fill: number, line: number, lw: number) {
  const [x1, y1, x2, y2] = [s[0] * sc, -s[1] * sc, s[2] * sc, -s[3] * sc];
  const dx = x2 - x1, dy = y2 - y1, len = Math.hypot(dx, dy) || 1;
  const rr = r * sc, nx = -dy / len * rr, ny = dx / len * rr, ang = Math.atan2(dy, dx);
  g.moveTo(x1 + nx, y1 + ny).lineTo(x2 + nx, y2 + ny)
    .arc(x2, y2, rr, ang + Math.PI / 2, ang - Math.PI / 2, true)
    .lineTo(x1 - nx, y1 - ny)
    .arc(x1, y1, rr, ang - Math.PI / 2, ang + Math.PI / 2, true)
    .closePath().fill(fill).stroke({ width: lw, color: line, alignment: 0.5 });
}

export interface DrawOpts {
  color: number; accent: number; scale: number; time: number; // scale = px per cm; time in s for flicker
  detached: boolean[]; flash: number; // flash 0..1 white overlay on struck robot
}
