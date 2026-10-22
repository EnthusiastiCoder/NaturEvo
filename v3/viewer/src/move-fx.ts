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
