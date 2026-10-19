// Sudden-death mechanic helpers: wall positions and torso bleed as pure functions of the tick (mirrors sim Duel::bounds / bleed).
import type { Fight } from './data';

const WALL_INSET_CM = 50; // walls start 0.5 m inside each arena edge

/** x (cm) of the left and right arena walls at `tick`. Static when the log has no sudden_death block. */
export function wallsAt(fight: Fight, tick: number): { lo: number; hi: number } {
  const w = fight.arena.width, sd = fight.suddenDeath;
  let shrink = 0;
  if (sd && tick > sd.startTick) {
    const max = Math.max(0, (w - 2 * WALL_INSET_CM - sd.minWidth) / 2);
    shrink = Math.min(max, (sd.speedCm * (tick - sd.startTick)) / 60);
  }
  return { lo: WALL_INSET_CM + shrink, hi: w - WALL_INSET_CM - shrink };
}

export function suddenDeathActive(fight: Fight, tick: number): boolean {
  return !!fight.suddenDeath && tick >= fight.suddenDeath.startTick;
}

/** Torso bleed in hp/s at `tick` (0 before sudden death). */
export function bleedAt(fight: Fight, tick: number): number {
  const sd = fight.suddenDeath;
  if (!sd || tick < sd.startTick) return 0;
  return sd.bleedBase * (1 + sd.bleedGrowth * ((tick - sd.startTick) / 60));
}
