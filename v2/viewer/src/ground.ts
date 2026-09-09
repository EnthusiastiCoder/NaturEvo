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
