export interface DamageStyle {
  scratches: boolean; desaturate: number; cracks: boolean; limp: boolean; detached: boolean;
  dents: boolean; darken: number; smoke: number; sparking: boolean;
}

/** hp in percent 0..100. >=75 clean, <75 dents/scratches, <50 darkened + cracks + sparking + smoke, 0 limp, destroyed detaches. */
export function partDamageStyle(hp: number, destroyed = false): DamageStyle {
  const low = hp < 50 && !destroyed;
  return {
    scratches: hp < 75,
    desaturate: hp < 50 ? Math.min(1, 0.3 + 0.7 * (1 - hp / 50)) : 0,
    cracks: hp < 50,
    limp: hp <= 0,
    detached: destroyed,
    dents: hp < 75,
    darken: hp < 50 ? Math.min(0.6, 0.15 + 0.45 * (1 - hp / 50)) : 0,
    smoke: low ? Math.min(1, 0.3 + 0.7 * (1 - hp / 50)) : 0,
    sparking: low,
  };
}
