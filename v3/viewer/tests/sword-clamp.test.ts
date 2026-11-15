import { describe, it, expect } from 'vitest';
import { clampSegAboveGround } from '../src/robot-draw';

describe('clampSegAboveGround', () => {
  it('leaves above-ground segments untouched', () => {
    expect(clampSegAboveGround([0, 10, 5, 20], 2.5)).toEqual([0, 10, 5, 20]);
  });
  it('clips the dipping tip along the line', () => {
    const s = clampSegAboveGround([0, 10, 10, -10], 0);
    expect(s[3]).toBeCloseTo(0); expect(s[2]).toBeCloseTo(5);
  });
  it('flattens a fully buried segment onto the floor', () => {
    expect(clampSegAboveGround([0, -5, 4, -9], 2.5)).toEqual([0, 2.5, 4, 2.5]);
  });
});
