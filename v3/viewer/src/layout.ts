/** Largest stage with the given aspect that fits the window. */
export function fitStage(winW: number, winH: number, stageW: number, stageH: number) {
  const scale = Math.max(0.01, Math.min(winW / stageW, winH / stageH));
  return { w: Math.floor(stageW * scale), h: Math.floor(stageH * scale), scale };
}

export interface Cam { scale: number; x: number; y: number }
type V = [number, number];
const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));

/** Zoom on the fighters (min view ~640 px wide), never showing outside the arena. Returns world container transform. */
export function fightCamera(arena: { w: number; h: number }, a: V, b: V, stage: { w: number; h: number }): Cam {
  const aspect = stage.w / stage.h;
  const vw = Math.max(Math.abs(a[0] - b[0]) + 360, (Math.abs(a[1] - b[1]) + 280) * aspect, 870);
  const cover = Math.max(stage.w / arena.w, stage.h / arena.h);
  const scale = Math.max(stage.w / vw, cover);
  const cx = (a[0] + b[0]) / 2, cy = (a[1] + b[1]) / 2;
  return { scale, x: clamp(stage.w / 2 - cx * scale, stage.w - arena.w * scale, 0), y: clamp(stage.h / 2 - cy * scale, stage.h - arena.h * scale, 0) };
}
