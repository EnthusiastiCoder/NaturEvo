// Vector-only robot drawing: every part is a Graphics capsule. Endpoints come straight from the log.
import { Graphics } from 'pixi.js';
import type { FighterFrame, Seg } from './data';
import { partDamageStyle } from './damage-style';

// order: Head Torso Pelvis UArmL UArmR FArmL FArmR ThighL ThighR ShinL ShinR FootL FootR
export const HEAD_R_CM = 23, TORSO_R_CM = 30, NECK_R_CM = 9;
const RADIUS_CM = [HEAD_R_CM, TORSO_R_CM, 26, 11, 11, 10, 10, 15, 15, 12, 12, 9, 9];
const BACK = new Set([3, 5, 7, 9, 11]); // left limbs drawn darker (far side)
export const PART_Z = [8, 5, 4, 2, 6, 3, 7, 1, 2, 1, 2, 0, 1];

export function shade(hex: number, f: number): number {
  const r = Math.round(((hex >> 16) & 255) * f), g = Math.round(((hex >> 8) & 255) * f), b = Math.round((hex & 255) * f);
  return (Math.min(255, r) << 16) | (Math.min(255, g) << 8) | Math.min(255, b);
}
export function mixWhite(hex: number, k: number): number {
  const m = (c: number) => Math.round(c + (255 - c) * k);
  return (m((hex >> 16) & 255) << 16) | (m((hex >> 8) & 255) << 8) | m(hex & 255);
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

/** Shield plate drawn across the forearm axis (perpendicular), centred near the hand. */
export function shieldPlate(fa: Seg, along = 0.62, len = 70): Seg {
  const dx = fa[2] - fa[0], dy = fa[3] - fa[1], l = Math.hypot(dx, dy) || 1;
  const cx = fa[0] + dx * along, cy = fa[1] + dy * along, nx = -dy / l * len / 2, ny = dx / l * len / 2;
  return [cx - nx, cy - ny, cx + nx, cy + ny];
}
/** Visual neck: from the torso's upper end to the head centre. */
export function neckSeg(torso: Seg, head: Seg): Seg {
  const top = torso[1] >= torso[3] ? [torso[0], torso[1]] : [torso[2], torso[3]];
  return [top[0], top[1], (head[0] + head[2]) / 2, (head[1] + head[3]) / 2];
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
  partFlash?: number[]; // per-part 0..1 white flash of the struck part
}

/** Draw one fighter into g (world px = cm * scale, y flipped; caller positions the container). */
export function drawRobot(g: Graphics, f: FighterFrame, o: DrawOpts): void {
  g.clear();
  const sc = o.scale, lw = Math.max(1.5, Math.min(2, 1.5 * sc * 2));
  const order = PART_Z.map((z, i) => [z, i]).sort((a, b) => a[0] - b[0]).map((p) => p[1]);
  for (const i of order) {
    if (o.detached[i]) continue;
    if (i === 0 && !o.detached[1]) capsule(g, neckSeg(f.parts[1], f.parts[0]), NECK_R_CM, sc, shade(o.color, 0.55), shade(o.color, 0.3), lw);
    const hp = f.hp[i];
    const st = partDamageStyle(hp, false);
    if (st.limp && Math.floor(o.time * 14) % 3 === 0) continue; // flicker
    let fill = BACK.has(i) ? shade(o.color, 0.62) : o.color;
    fill = desat(fill, st.desaturate);
    fill = shade(fill, 1 - st.darken);
    if (o.flash > 0) fill = shade(fill, 1 + o.flash * 0.5);
    const pf = o.partFlash?.[i] ?? 0;
    if (pf > 0) fill = mixWhite(fill, pf);
    capsule(g, f.parts[i], RADIUS_CM[i], sc, fill, shade(o.color, 0.35), lw);
    drawDamage(g, f.parts[i], RADIUS_CM[i], sc, st, i, o.time);
    if (i === 0) { // visor
      const p = f.parts[0], vx = (p[0] + p[2]) / 2 + f.facing * 10, vy = (p[1] + p[3]) / 2 + 3;
      g.roundRect(vx * sc - 7 * sc, -vy * sc - 3 * sc, 14 * sc, 6 * sc, 2).fill({ color: o.accent, alpha: 0.95 });
    }
    if (i === 1) { // core glow
      const p = f.parts[1], cx = (p[0] + p[2]) / 2, cy = (p[1] + p[3]) / 2 + 8;
      g.circle(cx * sc, -cy * sc, 7 * sc).fill({ color: o.accent, alpha: 0.35 });
      g.circle(cx * sc, -cy * sc, 3.5 * sc).fill({ color: o.accent, alpha: 0.95 });
    }
  }
  // shield plate then sword
  if (f.hp[5] > 0 && !o.detached[5]) {
    capsule(g, shieldPlate(f.parts[5]), 5, sc, shade(o.color, 0.8), o.accent, lw);
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
  // dent: a dark notch with a light rim
  g.moveTo((mx - r * 0.5) * sc, -(my + r * 0.3) * sc).lineTo((mx + r * 0.2) * sc, -(my - r * 0.1) * sc).stroke({ width: w * 1.6, color: 0x000000, alpha: 0.5 });
  g.moveTo((mx - r * 0.5) * sc, -(my + r * 0.3 + 3) * sc).lineTo((mx + r * 0.2) * sc, -(my - r * 0.1 + 3) * sc).stroke({ width: w, color: 0xffffff, alpha: 0.25 });
  if (!st.cracks) return;
  g.moveTo(mx * sc, -(my + r * 0.7) * sc).lineTo((mx + r * 0.3) * sc, -my * sc).lineTo((mx - r * 0.2) * sc, -(my - r * 0.6) * sc)
    .stroke({ width: w * 1.8, color: 0x0a0a0a, alpha: 0.9 });
  g.moveTo((mx + r * 0.3) * sc, -my * sc).lineTo((mx + r * 0.7) * sc, -(my + r * 0.2) * sc).stroke({ width: w * 1.2, color: 0x0a0a0a, alpha: 0.8 });
  const f = Math.floor(time * 12 + i * 3);
  if (st.sparking && f % 3 === 0) { // spark leak, flickering
    const a = (f * 2.4) % 6.28, sx = mx + r * 0.3, sy = my;
    g.moveTo(sx * sc, -sy * sc).lineTo((sx + Math.cos(a) * 10) * sc, -(sy + Math.sin(a) * 10) * sc).stroke({ width: Math.max(1.5, 1.5 * sc), color: 0xffe9a0, alpha: 0.95 });
    g.circle(sx * sc, -sy * sc, 3 * sc).fill({ color: 0xffd36a, alpha: 0.95 });
  }
  if (st.smoke > 0) { // three rising puffs, phase-offset
    for (let k = 0; k < 3; k++) {
      const ph = (time * 0.9 + k / 3 + i * 0.13) % 1;
      g.circle((mx + (k - 1) * r * 0.3 + Math.sin(ph * 6 + k) * 5) * sc, -(my + r + ph * 45) * sc, (4 + ph * 8) * sc * (0.6 + st.smoke * 0.6)).fill({ color: 0x3a3a3a, alpha: 0.5 * st.smoke * (1 - ph) });
    }
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
