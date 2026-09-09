// Shaded jungle ground painted to a canvas: biome blend by zone, value-noise shading, dapple shadows.
export const TILE_COLORS: Record<string, [number, number, number]> = {
  '.': [118, 96, 66], ',': [58, 122, 52], '#': [92, 92, 100], C: [150, 120, 80],
  R: [96, 110, 70], F: [100, 84, 60], T: [44, 98, 44], W: [48, 110, 150],
};
const ZONE_TINT: [number, number, number][] = [[0, 10, 0], [6, 0, -6], [14, -8, -14], [20, -18, -10]];
