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

/** Clip a segment (cm, y up) so no part of it is below minY: the end that dips is pulled back along the line. */
export function clampSegAboveGround(s: Seg, minY: number): Seg {
  const [x1, y1, x2, y2] = s;
  if (y1 >= minY && y2 >= minY) return s;
  if (y1 < minY && y2 < minY) return [x1, Math.max(y1, minY), x2, Math.max(y2, minY)];
  const k = (minY - y1) / (y2 - y1);
  const cx = x1 + (x2 - x1) * k;
  return y1 < minY ? [cx, minY, x2, y2] : [x1, y1, cx, minY];
}

export interface DrawOpts {
  color: number; accent: number; scale: number; time: number; // scale = px per cm; time in s for flicker
  detached: boolean[]; flash: number; // flash 0..1 white overlay on struck robot
}

/** Draw one fighter into g (world px = cm * scale, y flipped; caller positions the container). */
export function drawRobot(g: Graphics, f: FighterFrame, o: DrawOpts): void {
  g.clear();
  const sc = o.scale, lw = Math.max(1.5, Math.min(2, 1.5 * sc * 2));
  const order = PART_Z.map((z, i) => [z, i]).sort((a, b) => a[0] - b[0]).map((p) => p[1]);
  const swordBack = f.facing; void swordBack;
  for (const i of order) {
    if (o.detached[i]) continue;
    const hp = f.hp[i];
    const st = partDamageStyle(hp, false);
    if (st.limp && Math.floor(o.time * 14) % 3 === 0) continue; // flicker
    let fill = BACK.has(i) ? shade(o.color, 0.62) : o.color;
    fill = desat(fill, st.desaturate);
    if (o.flash > 0) fill = shade(fill, 1 + o.flash * 1.2);
    capsule(g, f.parts[i], RADIUS_CM[i], sc, fill, shade(o.color, 0.35), lw);
    drawDamage(g, f.parts[i], RADIUS_CM[i], sc, st, i, o.time);
    if (i === 0) { // visor
      const p = f.parts[0], vx = (p[0] + p[2]) / 2 + f.facing * 6, vy = (p[1] + p[3]) / 2 + 2;
      g.roundRect(vx * sc - 5 * sc, -vy * sc - 2 * sc, 10 * sc, 4 * sc, 2).fill({ color: o.accent, alpha: 0.95 });
    }
    if (i === 1) { // core glow
      const p = f.parts[1], cx = (p[0] + p[2]) / 2, cy = (p[1] + p[3]) / 2 + 8;
      g.circle(cx * sc, -cy * sc, 7 * sc).fill({ color: o.accent, alpha: 0.35 });
      g.circle(cx * sc, -cy * sc, 3.5 * sc).fill({ color: o.accent, alpha: 0.95 });
    }
  }
  // shield plate then sword
  if (f.hp[5] > 0 && !o.detached[5]) {
    capsule(g, f.shield, 5, sc, shade(o.color, 0.8), o.accent, lw);
  }
  if (f.hp[6] > 0 && !o.detached[6]) {
    const s = clampSegAboveGround(f.sword, 2.5); // half blade width: tip never pokes below the ground line
    g.moveTo(s[0] * sc, -s[1] * sc).lineTo(s[2] * sc, -s[3] * sc).stroke({ width: 5 * sc, color: 0x9aa7b3, cap: 'round' });
    g.moveTo(s[0] * sc, -s[1] * sc).lineTo(s[2] * sc, -s[3] * sc).stroke({ width: Math.max(1.5, 1.5 * sc), color: 0xffffff, cap: 'round' });
  }
}

function drawDamage(g: Graphics, s: Seg, r: number, sc: number, st: ReturnType<typeof partDamageStyle>, i: number, time: number) {
  if (!st.scratches) return;
  const mx = (s[0] + s[2]) / 2, my = (s[1] + s[3]) / 2;
  const w = Math.max(1, sc);
  g.moveTo((mx - r * 0.5) * sc, -(my + r * 0.3) * sc).lineTo((mx + r * 0.2) * sc, -(my - r * 0.1) * sc).stroke({ width: w, color: 0x000000, alpha: 0.4 });
  if (st.cracks) {
    g.moveTo(mx * sc, -(my + r * 0.6) * sc).lineTo((mx + r * 0.3) * sc, -my * sc).lineTo((mx - r * 0.2) * sc, -(my - r * 0.5) * sc)
      .stroke({ width: w * 1.3, color: 0x111111, alpha: 0.85 });
    if (Math.floor(time * 12 + i * 3) % 4 === 0) { // spark leak
      g.circle((mx + r * 0.3) * sc, -my * sc, 2 * sc).fill({ color: 0xffd36a, alpha: 0.9 });
    }
    if (st.desaturate > 0.6) g.circle(mx * sc, -(my + r + 6) * sc - (time * 30 % 12) * sc, 3 * sc).fill({ color: 0x555555, alpha: 0.35 });
  }
}

/** Debris body for a destroyed part: simple spinning capsule falling under gravity. */
export function debrisPose(seg: Seg, since: number, i: number): Seg {
  const t = Math.max(0, since), fall = Math.max(0, seg[1] - 9 - 490 * t * t), fall2 = Math.max(0, seg[3] - 9 - 490 * t * t);
  const mx = (seg[0] + seg[2]) / 2 + (i % 2 ? 30 : -30) * t, my = (fall + fall2) / 2;
  const a = Math.atan2(seg[3] - seg[1], seg[2] - seg[0]) + t * (i % 2 ? 6 : -6);
  const h = Math.hypot(seg[2] - seg[0], seg[3] - seg[1]) / 2;
  return [mx - Math.cos(a) * h, Math.max(0, my - Math.sin(a) * h), mx + Math.cos(a) * h, Math.max(0, my + Math.sin(a) * h)];
}
