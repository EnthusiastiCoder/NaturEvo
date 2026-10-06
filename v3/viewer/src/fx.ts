import type { FightEvent } from './data';
export interface SparkSpec { kind: 'cone' | 'radial' | 'ring'; count: number; color: number; spread: number; dirAngle: number }

export function hitstopTicks(events: FightEvent[], t: number): number {
  const e = events.find((x) => x.kind === 'hitstop' && x.t === t);
  return e ? e.ticks : 0;
}
