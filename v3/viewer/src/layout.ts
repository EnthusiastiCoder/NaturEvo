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

/** Campaign camera: zoomed around the hero (like v1), clamped to the map. */
export function mapCamera(map: { w: number; h: number }, hero: V, stage: { w: number; h: number }, zoom = 1.6): Cam {
  const scale = Math.max(zoom, stage.w / map.w, stage.h / map.h);
  return { scale, x: clamp(stage.w / 2 - hero[0] * scale, stage.w - map.w * scale, 0), y: clamp(stage.h / 2 - hero[1] * scale, stage.h - map.h * scale, 0) };
}

export const VISUAL_MIN_DIST = 2.2;
/** Visual-only separation so melee fighters never fully overlap (distances in tiles). */
export function separate(a: V, b: V, minDist: number): [V, V] {
  const dx = b[0] - a[0], dy = b[1] - a[1];
  const d = Math.hypot(dx, dy);
  if (d >= minDist) return [a, b];
  const ux = d > 1e-6 ? dx / d : 1, uy = d > 1e-6 ? dy / d : 0;
  const push = (minDist - d) / 2;
  return [[a[0] - ux * push, a[1] - uy * push], [b[0] + ux * push, b[1] + uy * push]];
}

export interface StateLabel { text: string; color: number }
export function stateLabel(state: string): StateLabel | null {
  switch (state) {
    case 'Windup': return { text: 'WINDUP', color: 0xffd166 };
    case 'Active': return { text: 'STRIKE!', color: 0xff3b30 };
    case 'Blocking': return { text: 'BLOCK', color: 0x6cc4ff };
    case 'Dodging': return { text: 'DODGE', color: 0xc9a7ff };
    case 'Recovery': return { text: 'RECOVER', color: 0xffa8a8 };
    case 'GuardRaise': return { text: 'GUARD UP', color: 0x9bd7ff };
    case 'GuardRecover': return { text: 'GUARD DOWN', color: 0x8fb3cc };
    case 'Staggered': return { text: 'STAGGER', color: 0xffa94d };
    case 'Stunned': return { text: 'STUN', color: 0xffd43b };
    case 'Knocked': return { text: 'KNOCKED', color: 0xffffff };
    default: return null;
  }
}
export const STAGE_W = 1920, STAGE_H = 1080;

/** Backing-store plan: render at the displayed size times dpr, and scale the logical stage into it (no css resampling). */
export function displayPlan(hostW: number, hostH: number, stageW: number, stageH: number, dpr: number) {
  const f = fitStage(hostW, hostH, stageW, stageH);
  return { cssW: f.w, cssH: f.h, resolution: dpr, stageScale: f.w / stageW };
}

/** Whole arena inside the stage, centred (split-screen halves). */
export function fitCamera(arena: { w: number; h: number }, stage: { w: number; h: number }): Cam {
  const scale = Math.min(stage.w / arena.w, stage.h / arena.h);
  return { scale, x: (stage.w - arena.w * scale) / 2, y: (stage.h - arena.h * scale) / 2 };
}

/** Fighter-related overlay scale: keeps fighters at least ~1.5 screen px per sprite px when the camera is zoomed out. */
export function uiScale(camScale: number): number { return Math.max(1, 1.5 / Math.max(0.01, camScale)); }

/** Offsets (world px, y down, relative to the fighter's feet) of the HP bar and the state label; the label is always above the bar. */
export function hudOffsets(u: number): { barTop: number; labelBottom: number; statusRowTop: number; statusRowBottom: number } {
  return { barTop: -92 * u, labelBottom: -118 * u, statusRowTop: -114 * u, statusRowBottom: -100 * u };
}
