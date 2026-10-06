import type { Frame } from './data';
export interface Camera { scale: number; tx: number; ty: number }
const MARGIN = 28; // px, above the 20px the tests require
const PAD_CM = 40;

/** Frame both fighters (with swords) and the ground. screen = (x*scale+tx, ty-y*scale). */
export function cameraFor(frame: Frame, w: number, h: number): Camera {
  let minX = Infinity, maxX = -Infinity, maxY = 0;
  for (const g of frame.f) {
    for (const p of [...g.parts, g.sword, g.shield]) {
      minX = Math.min(minX, p[0], p[2]); maxX = Math.max(maxX, p[0], p[2]);
      maxY = Math.max(maxY, p[1], p[3]);
    }
  }
  const spanX = maxX - minX + 2 * PAD_CM;
  const spanY = maxY + PAD_CM; // ground at y=0
  const scale = Math.max(0.05, Math.min((w - 2 * MARGIN) / spanX, (h - 2 * MARGIN) / spanY, 3));
  const cx = (minX + maxX) / 2;
  return { scale, tx: w / 2 - cx * scale, ty: h - MARGIN - (h - 2 * MARGIN - maxY * scale) / 2 + 0 };
}

export function lerpCamera(a: Camera, b: Camera, k: number): Camera {
  return { scale: a.scale + (b.scale - a.scale) * k, tx: a.tx + (b.tx - a.tx) * k, ty: a.ty + (b.ty - a.ty) * k };
}
