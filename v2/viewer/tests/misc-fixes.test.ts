import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { sceneSize, splitCellSize } from '../src/record';
import { themeIconPath } from '../src/theme';
import { groundPixels, lowResSub } from '../src/ground';
import { STAGE_W, STAGE_H } from '../src/layout';

describe('record canvas', () => {
  it('is exactly 1920x1080 at resolution 1 in record mode, whatever the dpr', () => {
    expect(sceneSize(true, 2)).toEqual({ w: 1920, h: 1080, resolution: 1 });
    expect(sceneSize(false, 2).w).toBe(STAGE_W);
    expect(sceneSize(false, 3).resolution).toBe(2);
  });
  it('normal stage is 16:9', () => { expect(STAGE_W / STAGE_H).toBeCloseTo(16 / 9, 3); });
  it('split cells tile the full frame with no empty bands', () => {
    const c = splitCellSize(1920, 1080);
    expect(c).toEqual({ w: 960, h: 1080 });
    expect(c.w * 2).toBe(1920);
  });
});
describe('theme icon', () => {
  it('shows sun in dark theme (action: go light) and moon in light theme, distinct', () => {
    expect(themeIconPath('dark')).not.toBe(themeIconPath('light'));
    expect(themeIconPath('dark')).toContain('circle');
  });
});
describe('ground texture', () => {
  it('is dpr aware and keeps low-res pass modest', () => {
    expect(groundPixels(40, 40, 48, 0.8, 2)).toEqual({ w: 3840, h: 3072 });
    expect(groundPixels(40, 40, 48, 0.8, 1)).toEqual({ w: 1920, h: 1536 });
    expect(lowResSub(48)).toBeGreaterThanOrEqual(8);
  });
});
describe('css', () => {
  const css = readFileSync(resolve(__dirname, '../src/style.css'), 'utf8');
  it('stage and timeline track follow the theme', () => {
    expect(css).toMatch(/--stage:/);
    expect(css).toMatch(/#stage[^}]*background:\s*var\(--stage\)/);
    expect(css).toMatch(/::-webkit-slider-runnable-track/);
    expect(css).toMatch(/::-moz-range-track/);
  });
});
