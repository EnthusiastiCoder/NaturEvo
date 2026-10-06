export interface DamageStyle { scratches: boolean; desaturate: number; cracks: boolean; limp: boolean; detached: boolean }

/** hp in percent 0..100. >=75 clean, <75 scratches, <50 desaturated + cracks, 0 limp, destroyed detaches. */
export function partDamageStyle(hp: number, destroyed = false): DamageStyle {
  return {
    scratches: hp < 75,
    desaturate: hp < 50 ? Math.min(1, 0.3 + 0.7 * (1 - hp / 50)) : 0,
    cracks: hp < 50,
    limp: hp <= 0,
    detached: destroyed,
  };
}
