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
