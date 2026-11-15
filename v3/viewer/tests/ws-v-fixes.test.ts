// Final visual verification defects (plans/reports/2026-10-24_ws-v-viewer-verification.md).
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { strokeArcFrom } from '../src/move-fx';

describe('OPEN ring arc', () => {
  it('moves to the arc start before arc() so no stray line is drawn', () => {
    const calls: string[] = [];
    const g: any = { moveTo: (x: number, y: number) => (calls.push(`m ${x.toFixed(1)} ${y.toFixed(1)}`), g), arc: () => (calls.push('arc'), g) };
    strokeArcFrom(g, 10, 20, 5, -Math.PI / 2, 0);
    expect(calls).toEqual(['m 10.0 15.0', 'arc']);
  });
});

describe('result card', () => {
  it('sits at the bottom, clear of the top-corner HUD panels and the K.O. banner', () => {
    const css = readFileSync(resolve(__dirname, '../src/style.css'), 'utf8');
    const rule = [...css.matchAll(/\.fv-result\s*\{[^}]*\}/g)].map((m) => m[0]).pop()!;
    expect(rule).toMatch(/bottom\s*:\s*12px/);
    expect(rule).not.toMatch(/top\s*:/);
  });
});
