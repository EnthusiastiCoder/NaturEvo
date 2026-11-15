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

describe('status tokens', () => {
  it('style.css defines one --fx-* colour token per status', () => {
    const c = css();
    for (const k of ['knock', 'stagger', 'stun', 'confused', 'slowed', 'bleed', 'poison']) expect(c).toMatch(new RegExp(`--fx-${k}\\s*:`));
  });
});

describe('no scroll', () => {
  it('page cannot scroll: body overflow hidden and viewport height units', () => {
    const c = css();
    expect(c).toMatch(/overflow\s*:\s*hidden/);
    expect(c).toMatch(/100(d|s)?vh|100%/);
  });
  for (const [w, h] of [[1282, 642], [1920, 1080]] as const) {
    it(`stage fits ${w}x${h} without exceeding the window`, () => {
      const s = fitStage(w, h, 1536, 1024);
      expect(s.w).toBeLessThanOrEqual(w);
      expect(s.h).toBeLessThanOrEqual(h);
      expect(s.scale).toBeGreaterThan(0);
    });
  }
});

describe('accessibility', () => {
  it('every icon-only button has an aria-label', () => {
    const buttons = html().match(/<button\b[^>]*>[\s\S]*?<\/button>/g) ?? [];
    expect(buttons.length).toBeGreaterThan(0);
    for (const b of buttons) {
      const textOnly = b.replace(/<svg[\s\S]*?<\/svg>/g, '').replace(/<[^>]+>/g, '').trim();
      if (b.includes('<svg') && textOnly === '') expect(b, b).toMatch(/aria-label="[^"]+"/);
    }
  });
  it('has a theme toggle and info buttons with tooltips', () => {
    const h = html();
    expect(h).toMatch(/id="theme-toggle"/);
    expect(h).toMatch(/title="[^"]+"|data-tooltip="[^"]+"/);
  });
  it('uses inline SVG icons, no icon fonts', () => {
    expect(html()).toContain('<svg');
    expect(html()).not.toMatch(/font-?awesome|material-icons/i);
  });
});
