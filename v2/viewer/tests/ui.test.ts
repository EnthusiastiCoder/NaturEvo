// Static checks on v2/viewer/index.html and src/style.css (no DOM dependency; only pixi/vite/vitest allowed).
// Assumed: src/theme.ts exports DEFAULT_ACCENT, nextTheme(t), applyTheme(root,t), setAccent(root,hex),
// src/layout.ts exports fitStage(winW, winH, stageW, stageH) -> {w,h,scale}.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { DEFAULT_ACCENT, nextTheme, applyTheme, setAccent } from '../src/theme';
import { fitStage } from '../src/layout';

const root = resolve(__dirname, '..');
const html = () => readFileSync(resolve(root, 'index.html'), 'utf8');
const css = () => readFileSync(resolve(root, 'src/style.css'), 'utf8');

describe('theme', () => {
  it('toggles light <-> dark', () => {
    expect(nextTheme('light')).toBe('dark');
    expect(nextTheme('dark')).toBe('light');
  });
  it('applyTheme sets data-theme', () => {
    const el = { dataset: {} as Record<string, string>, style: { setProperty() {} } } as any;
    applyTheme(el, 'dark');
    expect(el.dataset.theme).toBe('dark');
  });
  it('accent defaults to jungle green and is customisable', () => {
    expect(DEFAULT_ACCENT).toMatch(/^#(2|3|4)[0-9a-f]{1}[a-f0-9]{4}$/i);
    const r = parseInt(DEFAULT_ACCENT.slice(1, 3), 16), g = parseInt(DEFAULT_ACCENT.slice(3, 5), 16), b = parseInt(DEFAULT_ACCENT.slice(5, 7), 16);
    expect(g).toBeGreaterThan(r);
    expect(g).toBeGreaterThan(b);
    const set: Record<string, string> = {};
    setAccent({ style: { setProperty: (k: string, v: string) => { set[k] = v; } } } as any, '#ff8800');
    expect(set['--accent']).toBe('#ff8800');
  });
  it('one style system: CSS tokens for both themes and accent', () => {
    const c = css();
    expect(c).toContain('--accent');
    expect(c).toMatch(/\[data-theme=["']?dark["']?\]/);
    expect(c).toMatch(/\[data-theme=["']?light["']?\]|:root/);
    expect(html().match(/<link[^>]+stylesheet/g)?.length ?? 0).toBeLessThanOrEqual(1);
  });
});
