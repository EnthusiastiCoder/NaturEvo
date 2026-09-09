// Shaded jungle ground painted to a canvas: biome blend by zone, value-noise shading, dapple shadows.
export const TILE_COLORS: Record<string, [number, number, number]> = {
  '.': [118, 96, 66], ',': [58, 122, 52], '#': [92, 92, 100], C: [150, 120, 80],
  R: [96, 110, 70], F: [100, 84, 60], T: [44, 98, 44], W: [48, 110, 150],
};
const ZONE_TINT: [number, number, number][] = [[0, 10, 0], [6, 0, -6], [14, -8, -14], [20, -18, -10]];

export function hash2(x: number, y: number, s = 0): number {
  const h = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453123;
  return h - Math.floor(h);
}
function vnoise(x: number, y: number, s: number): number {
  const xi = Math.floor(x), yi = Math.floor(y), xf = x - xi, yf = y - yi;
  const u = xf * xf * (3 - 2 * xf), v = yf * yf * (3 - 2 * yf);
  const a = hash2(xi, yi, s), b = hash2(xi + 1, yi, s), c = hash2(xi, yi + 1, s), d = hash2(xi + 1, yi + 1, s);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}
export const fbm = (x: number, y: number, s: number) =>
  vnoise(x, y, s) * 0.55 + vnoise(x * 2.1, y * 2.1, s + 5) * 0.3 + vnoise(x * 4.3, y * 4.3, s + 9) * 0.15;

export interface GroundSpec { w: number; h: number; tile: (x: number, y: number) => string; zone: (x: number, y: number) => number; seed: number }

/** Paint the ground. ts = pixels per tile horizontally; pitch = vertical squash. */
export function paintGround(g: GroundSpec, ts: number, pitch: number): HTMLCanvasElement {
  const SUB = 4;
  const lw = g.w * SUB, lh = g.h * SUB;
  const low = document.createElement('canvas');
  low.width = lw; low.height = lh;
  const lc = low.getContext('2d')!;
  const img = lc.createImageData(lw, lh);
  for (let py = 0; py < lh; py++) {
    for (let px = 0; px < lw; px++) {
      const tx = Math.min(g.w - 1, Math.floor(px / SUB)), ty = Math.min(g.h - 1, Math.floor(py / SUB));
      // blend colour with the 4 nearest tile centres for soft biome edges
      const fx = (px + 0.5) / SUB - 0.5, fy = (py + 0.5) / SUB - 0.5;
      const x0 = Math.floor(fx), y0 = Math.floor(fy), ax = fx - x0, ay = fy - y0;
      let r = 0, gg = 0, b = 0;
      for (let k = 0; k < 4; k++) {
        const cx = Math.max(0, Math.min(g.w - 1, x0 + (k & 1))), cy = Math.max(0, Math.min(g.h - 1, y0 + (k >> 1)));
        const wgt = ((k & 1) ? ax : 1 - ax) * ((k >> 1) ? ay : 1 - ay);
        const t = g.tile(cx, cy);
        const base = TILE_COLORS[t === 'T' ? ',' : t === 'R' || t === 'F' ? '.' : t] ?? TILE_COLORS['.'];
        const z = ZONE_TINT[Math.min(3, g.zone(cx, cy))];
        r += (base[0] + z[0]) * wgt; gg += (base[1] + z[1]) * wgt; b += (base[2] + z[2]) * wgt;
      }
      const n = fbm(px / 5, py / 5, g.seed) - 0.5;
      const dapple = fbm(px / 11 + 40, py / 11, g.seed + 3) > 0.62 ? -22 : 0; // canopy light gaps
      const t = g.tile(tx, ty);
      const shade = 1 + n * 0.55 + dapple / 255;
      const wob = t === 'W' ? Math.sin(px * 0.9 + py * 1.3) * 6 : 0;
      const i = (py * lw + px) * 4;
      img.data[i] = Math.max(0, Math.min(255, r * shade + wob));
      img.data[i + 1] = Math.max(0, Math.min(255, gg * shade + wob));
      img.data[i + 2] = Math.max(0, Math.min(255, b * shade + wob));
      img.data[i + 3] = 255;
    }
  }
  lc.putImageData(img, 0, 0);
  const out = document.createElement('canvas');
  out.width = Math.round(g.w * ts); out.height = Math.round(g.h * ts * pitch);
  const c = out.getContext('2d')!;
  c.imageSmoothingEnabled = true; c.imageSmoothingQuality = 'high';
  c.drawImage(low, 0, 0, out.width, out.height);
  // detail: grass tufts, pebbles and water glints, deterministic
  const sx = ts, sy = ts * pitch;
  for (let y = 0; y < g.h; y++) for (let x = 0; x < g.w; x++) {
    const t = g.tile(x, y);
    if (t === 'W') {
      c.fillStyle = 'rgba(255,255,255,0.10)';
      for (let k = 0; k < 3; k++) c.fillRect((x + hash2(x, y, k)) * sx, (y + hash2(y, x, k + 7)) * sy, sx * 0.4, 1.2);
      continue;
    }
    const grassy = t === ',' || t === 'T';
    const count = grassy ? 6 : 2;
    for (let k = 0; k < count; k++) {
      const px = (x + hash2(x, y, k + 11)) * sx, py = (y + hash2(x, y, k + 31)) * sy;
      if (grassy) {
        c.strokeStyle = hash2(x, y, k) > 0.5 ? 'rgba(120,190,80,0.55)' : 'rgba(30,80,30,0.55)';
        c.lineWidth = 1.2; c.beginPath(); c.moveTo(px, py);
        c.lineTo(px + (hash2(k, x, y) - 0.5) * 4, py - 4 - hash2(y, k, x) * 4); c.stroke();
      } else { c.fillStyle = 'rgba(70,56,40,0.35)'; c.fillRect(px, py, 2, 1.4); }
    }
  }
  // soft vignette keeps focus on the centre (lower cognitive load)
  const vg = c.createRadialGradient(out.width / 2, out.height / 2, Math.min(out.width, out.height) * 0.35,
    out.width / 2, out.height / 2, Math.max(out.width, out.height) * 0.72);
  vg.addColorStop(0, 'rgba(0,0,0,0)'); vg.addColorStop(1, 'rgba(0,10,0,0.45)');
  c.fillStyle = vg; c.fillRect(0, 0, out.width, out.height);
  return out;
}

/** Tree texture with a baked soft shadow; variant 0..2. Anchor at bottom centre (48,108). */
export function paintTree(variant: number): HTMLCanvasElement {
  const c = document.createElement('canvas'); c.width = 96; c.height = 120;
  const x = c.getContext('2d')!;
  x.fillStyle = 'rgba(0,0,0,0.28)'; x.beginPath(); x.ellipse(52, 108, 34, 10, 0, 0, 7); x.fill();
  x.fillStyle = '#5b3d22'; x.fillRect(43, 62, 10, 46);
  x.fillStyle = '#7a5530'; x.fillRect(43, 62, 4, 46);
  const greens = ['#2c6b32', '#35803a', '#3f9444', '#52aa4f'];
  for (let i = 0; i < 4; i++) {
    x.fillStyle = greens[i];
    const r = [34, 29, 23, 15][i];
    x.beginPath(); x.ellipse(48 + (variant - 1) * 3 - i * 2, 56 - i * 11, r, r * 0.8, 0, 0, 7); x.fill();
  }
  return c;
}

/** Rock texture, anchored bottom centre (24,40). */
export function paintRock(variant: number): HTMLCanvasElement {
  const c = document.createElement('canvas'); c.width = 48; c.height = 48;
  const x = c.getContext('2d')!;
  x.fillStyle = 'rgba(0,0,0,0.3)'; x.beginPath(); x.ellipse(26, 40, 20, 6, 0, 0, 7); x.fill();
  x.fillStyle = '#6b7260'; x.beginPath(); x.moveTo(6, 40); x.lineTo(10 + variant * 2, 16); x.lineTo(28, 8); x.lineTo(42, 22); x.lineTo(44, 40); x.closePath(); x.fill();
  x.fillStyle = '#8a9280'; x.beginPath(); x.moveTo(10 + variant * 2, 16); x.lineTo(28, 8); x.lineTo(30, 24); x.lineTo(14, 30); x.closePath(); x.fill();
  return c;
}
