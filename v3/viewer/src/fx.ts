import type { FightEvent } from './data';
export interface SparkSpec { kind: 'cone' | 'radial' | 'ring'; count: number; color: number; spread: number; dirAngle: number }

export function hitstopTicks(events: FightEvent[], t: number): number {
  const e = events.find((x) => x.kind === 'hitstop' && x.t === t);
  return e ? e.ticks : 0;
}

export function sparkSpec(e: FightEvent): SparkSpec | null {
  switch (e.kind) {
    case 'hit':
      return { kind: 'cone', count: Math.round(Math.min(60, 6 + e.impact * 0.2)), color: 0xffa83a, spread: Math.PI / 3, dirAngle: Math.atan2(e.dy, e.dx) };
    case 'clash': {
      const perfect = e.type === 'perfect_parry';
      return { kind: 'radial', count: perfect ? 40 : 28, color: perfect ? 0xfff2b0 : 0xffe9a8, spread: 2 * Math.PI, dirAngle: Math.atan2(e.dy, e.dx) };
    }
    case 'block':
      return { kind: 'ring', count: 10, color: 0x7fd4ff, spread: 2 * Math.PI, dirAngle: 0 };
    default:
      return null;
  }
}
