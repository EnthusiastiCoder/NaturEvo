import type { FightEvent } from './data';
import { hitFx } from './impact';
export interface SparkSpec { kind: 'cone' | 'radial' | 'ring'; count: number; color: number; spread: number; dirAngle: number }

export function hitstopTicks(events: FightEvent[], t: number): number {
  const e = events.find((x) => x.kind === 'hitstop' && x.t === t);
  return e ? e.ticks : 0;
}

export function sparkSpec(e: FightEvent): SparkSpec | null {
  switch (e.kind) {
    case 'hit':
      return { kind: 'cone', count: hitFx(e.impact).sparks, color: 0xffa83a, spread: Math.PI / 3, dirAngle: Math.atan2(e.dy, e.dx) };
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

/** Screen shake amplitude in px for an event (0 if none). */
export function shakeFor(e: FightEvent): number {
  return e.kind === 'hit' ? hitFx(e.impact).shake : e.kind === 'clash' ? (e.type === 'clash' ? 2 : 3) : e.kind === 'block' ? 2 : e.kind === 'ko' ? 14 : 0;
}

export interface Particle { x: number; y: number; vx: number; vy: number; life: number; max: number; color: number; size: number }

/** Deterministic particle burst (seeded by tick) for a spark spec at world point (cm). */
export function spawnParticles(spec: SparkSpec, x: number, y: number, seed: number): Particle[] {
  let s = (seed * 2654435761) >>> 0;
  const rnd = () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296);
  const out: Particle[] = [];
  for (let i = 0; i < spec.count; i++) {
    const a = spec.kind === 'ring' ? (i / spec.count) * 2 * Math.PI : spec.dirAngle + (rnd() - 0.5) * spec.spread;
    const sp = spec.kind === 'ring' ? 120 : 150 + rnd() * 350;
    out.push({ x, y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, life: 0, max: 0.2 + rnd() * 0.3, color: spec.color, size: 1 + rnd() * 1.5 });
  }
  return out;
}

/** Events fired within `windowTicks` before `tick` (inclusive), with age in seconds. Used so scrubbing to a hit shows its sparks. */
export function recentEvents(events: FightEvent[], tick: number, windowTicks = 18): { e: FightEvent; age: number }[] {
  return events.filter((e) => e.t <= tick && e.t >= tick - windowTicks).map((e) => ({ e, age: (tick - e.t) / 60 }));
}

/** Advance particles by `age` seconds (same integrator as the live renderer) and drop expired ones. */
export function ageParticles(ps: Particle[], age: number): Particle[] {
  const out: Particle[] = [];
  for (const p of ps) {
    const q = { ...p };
    for (let t = 0; t < age; t += 1 / 60) {
      const dt = Math.min(1 / 60, age - t);
      q.x += q.vx * dt; q.y += q.vy * dt; q.vy -= 600 * dt; q.life += dt;
    }
    if (q.life < q.max) out.push(q);
  }
  return out;
}
