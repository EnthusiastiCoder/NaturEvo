import { describe, it, expect } from 'vitest';
import { HEAD_R_CM, TORSO_R_CM, shieldPlate, neckSeg } from '../src/robot-draw';

describe('humanoid proportions', () => {
  it('head is large relative to torso', () => {
    expect(HEAD_R_CM).toBeGreaterThanOrEqual(21);
    expect(HEAD_R_CM / TORSO_R_CM).toBeGreaterThanOrEqual(0.7);
  });
  it('shield plate is perpendicular to the forearm', () => {
    const fa: [number, number, number, number] = [0, 100, 30, 170];
    const s = shieldPlate(fa);
    const dot = (fa[2] - fa[0]) * (s[2] - s[0]) + (fa[3] - fa[1]) * (s[3] - s[1]);
    expect(Math.abs(dot)).toBeLessThan(1e-6);
    expect(Math.hypot(s[2] - s[0], s[3] - s[1])).toBeGreaterThan(40);
  });
  it('neck links torso top to head', () => {
    const n = neckSeg([0, 100, 0, 200], [5, 230, 5, 240]);
    expect(n[1]).toBe(200);
    expect(n[0]).toBe(0);
    expect(n[3]).toBeCloseTo(235);
  });
});
