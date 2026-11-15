// Assumed module: src/damage-style.ts exports partDamageStyle(hp: number /*0..100 percent*/, destroyed?: boolean)
//   -> {scratches: boolean, desaturate: number /*0..1*/, cracks: boolean, limp: boolean, detached: boolean}
import { describe, it, expect } from 'vitest';
import { partDamageStyle } from '../src/damage-style';

describe('partDamageStyle', () => {
  it.each([100, 75])('hp %i is clean', (hp) => {
    expect(partDamageStyle(hp, false)).toMatchObject({
      scratches: false, desaturate: 0, cracks: false, limp: false, detached: false,
    });
  });
  it('hp 74 has scratches only', () => {
    const s = partDamageStyle(74, false);
    expect(s.scratches).toBe(true);
    expect(s.cracks).toBe(false);
    expect(s.desaturate).toBe(0);
    expect(s.limp).toBe(false);
  });
  it('hp 49 desaturates and cracks', () => {
    const s = partDamageStyle(49, false);
    expect(s.scratches).toBe(true);
    expect(s.desaturate).toBeGreaterThan(0);
    expect(s.desaturate).toBeLessThanOrEqual(1);
    expect(s.cracks).toBe(true);
    expect(s.limp).toBe(false);
  });
  it('desaturation grows as hp falls', () => {
    expect(partDamageStyle(10, false).desaturate).toBeGreaterThan(partDamageStyle(49, false).desaturate);
  });
  it('hp 0 is limp, not detached', () => {
    const s = partDamageStyle(0, false);
    expect(s.limp).toBe(true);
    expect(s.detached).toBe(false);
  });
  it('destroyed (hp 0, destroyed) is detached', () => {
    const s = partDamageStyle(0, true);
    expect(s.detached).toBe(true);
  });
});
