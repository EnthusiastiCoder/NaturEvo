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
