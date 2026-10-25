// WS-E move FX: pure helpers (no Pixi/DOM). Times are log ticks, dt is seconds per tick.
import type { FightEvent, Frame, FighterFrame, Fight } from './data';

export interface Callout { text: string; x: number; y: number; alpha: number; color: string; kind: 'move' | 'punish' | 'counter' | 'guard_break' }
const KINDS: Record<string, { color: string; text: (e: FightEvent) => string }> = {
  move: { color: '#f5c542', text: (e) => String(e.name ?? 'Move') },
  punish: { color: '#ff6b4a', text: () => 'PUNISH!' },
  counter: { color: '#4ac8ff', text: () => 'COUNTER!' },
  guard_break: { color: '#c77dff', text: () => 'GUARD BREAK!' },
};
const CALLOUT_S = 0.6;

export function moveCallout(events: FightEvent[], frame: Frame, tNow: number, dt: number): Callout | null {
  const span = CALLOUT_S / dt;
  let best: FightEvent | null = null;
  for (const e of events) {
    if (!(e.kind in KINDS) || e.t > tNow || tNow >= e.t + span) continue;
    if (!best || e.t >= best.t) best = e;
  }
  if (!best) return null;
  const alpha = 1 - (tNow - best.t) / span;
  if (alpha <= 0) return null;
  const head = frame.f[best.who === 1 ? 1 : 0].parts[0];
  const k = KINDS[best.kind];
  return { text: k.text(best), x: (head[0] + head[2]) / 2, y: Math.max(head[1], head[3]) + 60, alpha, color: k.color, kind: best.kind as Callout['kind'] };
}

export interface PhaseStyle { glow: number; trail: boolean; dim: boolean; color: string }
export function phaseStyle(phase: 0 | 1 | 2 | 3, p: number): PhaseStyle {
  const q = Math.min(1, Math.max(0, p));
  switch (phase) {
    case 1: return { glow: Math.min(1, 0.15 + 0.85 * q), trail: false, dim: false, color: '#ffb000' };
    case 2: return { glow: 1, trail: true, dim: false, color: '#ffffff' };
    case 3: return { glow: 0, trail: false, dim: true, color: '#7a8594' };
    default: return { glow: 0, trail: false, dim: false, color: '#9aa4b2' };
  }
}

export interface OpenBadge { show: boolean; x: number; y: number; arc: number; riposte: boolean }
export function openBadge(f: FighterFrame): OpenBadge {
  const mv = f.mv;
  const head = f.parts[0];
  const show = !!mv?.open || f.state === 'staggered';
  const arc = mv ? Math.min(1, Math.max(0, 1 - mv.p)) : 1;
  return { show, x: (head[0] + head[2]) / 2, y: Math.max(head[1], head[3]) + 30, arc, riposte: !!mv?.rip };
}

const PHASES = ['Idle', 'Windup', 'Active', 'Recovery'];
export function hudMoveLine(f: FighterFrame): string {
  const mv = f.mv;
  if (!mv) return '';
  if (mv.phase === 0) return 'Idle';
  return `${mv.id} · ${PHASES[mv.phase]}`;
}

export interface GraceRow { key: 'whiff' | 'punish' | 'hits_per_swing' | 'jerk'; label: string; value: string; tooltip: string }
export function graceModel(result: Fight['result'], who: 0 | 1): { rows: GraceRow[]; usage: { name: string; count: number }[] } | null {
  const m = result.moves;
  if (!m) return null;
  const pct = (a?: number[]) => `${Math.round((a?.[who] ?? 0) * 100)}%`;
  const two = (a?: number[]) => (a?.[who] ?? 0).toFixed(2);
  const rows: GraceRow[] = [
    { key: 'whiff', label: 'Whiff', value: pct(m.whiff), tooltip: 'Share of swings that hit nothing' },
    { key: 'punish', label: 'Punish', value: pct(m.punish), tooltip: 'Share of opponent recoveries this fighter punished' },
    { key: 'hits_per_swing', label: 'Hits per swing', value: two(m.hits_per_swing), tooltip: 'Average hits landed per swing' },
    { key: 'jerk', label: 'Jerk', value: compactNum(m.jerk?.[who] ?? 0), tooltip: 'Motion roughness; lower is smoother' },
  ];
  const u = m.usage?.[who];
  const usage = Array.isArray(u) ? u.map((x: any) => ({ name: String(x.name), count: Number(x.count) }))
    : Object.entries(u ?? {}).map(([name, count]) => ({ name, count: Number(count) }));
  usage.sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
  return { rows, usage };
}

/** Stroke-ready arc: moves to the arc start first so the path never connects from a previous point. */
export function strokeArcFrom<G extends { moveTo(x: number, y: number): G; arc(x: number, y: number, r: number, a0: number, a1: number): G }>(g: G, cx: number, cy: number, r: number, a0: number, a1: number): G {
  return g.moveTo(cx + r * Math.cos(a0), cy + r * Math.sin(a0)).arc(cx, cy, r, a0, a1);
}

/** Compact number: 195614016 -> "195.6M", 1234 -> "1.23k", 0.12 -> "0.12". */
export function compactNum(v: number): string {
  const a = Math.abs(v);
  if (!Number.isFinite(v)) return '-';
  const f = (x: number, s: string) => `${x >= 100 ? x.toFixed(0) : x >= 10 ? x.toFixed(1) : x.toFixed(2)}${s}`;
  if (a >= 1e12) return f(v / 1e12, 'T');
  if (a >= 1e9) return f(v / 1e9, 'G');
  if (a >= 1e6) return f(v / 1e6, 'M');
  if (a >= 1e3) return f(v / 1e3, 'k');
  return v.toFixed(2);
}

/** "win_a" -> "Hero wins", "win_b" -> "Monster wins", "draw" -> "Draw". */
export function outcomeText(o: string): string {
  if (o === 'win_a') return 'Hero wins';
  if (o === 'win_b') return 'Monster wins';
  const s = o.replace(/_/g, ' ');
  return s.charAt(0).toUpperCase() + s.slice(1);
}

export interface LabelIn { id: string; x: number; y: number; age: number; w?: number; h?: number }
export interface LabelOut extends LabelIn { slot: number }
export interface Rect { x0: number; y0: number; x1: number; y1: number }

/** Screen rectangle the K.O. banner and its sub line can occupy (generous: covers the pop-in scale), mirroring scene.ts drawBanner. */
export function bannerRect(w: number, h: number): Rect {
  const fit = Math.min(1, w / 760) * (h < 700 ? 0.75 : 1) * 0.55, cy = h * 0.17;
  const hw = Math.max(260 * fit, 150 * fit * 1.6) + 8;
  return { x0: w / 2 - hw, x1: w / 2 + hw, y0: cy - 90 * fit * 1.2, y1: cy + 78 * fit + 30 * fit * 1.2 };
}

/** Deterministic label stack: newest `max` labels survive; each is lifted by `gap` steps until clear of every earlier-placed label within `reach` horizontally.
 *  With `avoid` (same space as the labels), a label whose box (w x h, centred) overlaps it drops to just below it and stacks downward from there. */
export function layoutLabels(items: LabelIn[], max = 4, gap = 30, reach = 160, avoid?: Rect): LabelOut[] {
  const keep = [...items].sort((a, b) => a.age - b.age || (a.id < b.id ? -1 : 1)).slice(0, max);
  const placed: LabelOut[] = [];
  const hits = (it: LabelIn, y: number) => !!avoid && it.x + (it.w ?? 0) / 2 > avoid.x0 && it.x - (it.w ?? 0) / 2 < avoid.x1 && y + (it.h ?? 0) / 2 > avoid.y0 && y - (it.h ?? 0) / 2 < avoid.y1;
  for (const it of keep) {
    let y = it.y, slot = 0, dir = -1;
    for (let n = 0; n < 12; n++) {
      if (hits(it, y)) { y = avoid!.y1 + (it.h ?? 0) / 2 + 1; dir = 1; }
      else if (placed.some((p) => Math.abs(p.x - it.x) < reach && Math.abs(p.y - y) < gap) && slot < max) { y += dir * gap; slot++; }
      else break;
    }
    placed.push({ ...it, y, slot });
  }
  return placed;
}
