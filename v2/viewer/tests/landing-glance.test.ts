// At-a-glance landing: labels, grouping, headline numbers, smoothing, charts, empty state.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { humanName, isScratch, groupRuns, defaultRun, formatDate } from '../src/run-labels';
import { smooth, tailMean } from '../src/aggregate';
import { headline } from '../src/headline';
import { lineChart, barChart, legendHtml, nearestIndex, tipText } from '../src/charts';
import { landingViewModel, renderLandingHtml, renderResultsHtml, EMPTY_COMMANDS } from '../src/landing';

const camp = (wins: number, losses: number) => ({ outcome: 'Died', wins, losses, encounters: wins + losses, perZone: { '1': { wins, losses } } });
const run = (name: string, date: string, extra: any = {}) => ({
  name, seed: 'n/a', date, generations: 200, permadeath: {}, keyResult: 'n/a',
  files: ['training.csv', 'fight_gen_9.json', 'fight_gen_199.json'], ...extra,
}) as any;

describe('run labels', () => {
  it('derives humane names', () => {
    expect(humanName('cd2_r1/s1')).toBe('cd2 round 1 / seed 1');
    expect(humanName('cd2/s3')).toBe('cd2 / seed 3');
    expect(humanName('main')).toBe('main');
  });
  it('detects scratch runs', () => {
    for (const n of ['tmp60', 't200s1', '_vv', 'x_tmp', 'a/tmp1']) expect(isScratch(n)).toBe(true);
    for (const n of ['cd2/s1', 'main', 'final/s2', 't200']) expect(isScratch(n)).toBe(false);
  });
  it('groups and sorts: campaign first, newest first, scratch apart', () => {
    const runs = [run('a', '2026-10-01T10:00:00Z'), run('b', '2026-10-02T10:00:00Z', { permadeath: { on: camp(1, 1) } }),
      run('c', '2026-10-03T10:00:00Z', { permadeath: { on: camp(1, 1) } }), run('tmp1', '2026-10-09T10:00:00Z')];
    const g = groupRuns(runs);
    expect(g.campaign.map((r) => r.name)).toEqual(['c', 'b']);
    expect(g.training.map((r) => r.name)).toEqual(['a']);
    expect(g.scratch.map((r) => r.name)).toEqual(['tmp1']);
    expect(defaultRun(runs)!.name).toBe('c');
    expect(defaultRun([run('a', '2026-10-01'), run('z', '2026-10-05')])!.name).toBe('z');
  });
  it('formats dates', () => {
    expect(formatDate('2026-10-23')).toMatch(/23/);
    expect(formatDate('n/a')).toBe('');
  });
});

describe('aggregate helpers', () => {
  it('smooth keeps length and flattens noise', () => {
    const ys = [0, 10, 0, 10, 0, 10, 0];
    const s = smooth(ys, 3);
    expect(s).toHaveLength(7);
    expect(Math.max(...s) - Math.min(...s)).toBeLessThan(10);
    expect(smooth([], 5)).toEqual([]);
  });
  it('tailMean averages the last n', () => {
    expect(tailMean([1, 2, 3, 4], 2)).toBe(3.5);
    expect(tailMean([], 3)).toBeNull();
  });
});

describe('headline', () => {
  const rows = Array.from({ length: 20 }, (_, i) => ({ gen: i, hof_wr_hero: 0.4, hof_wr_monster: 0.6, draw_rate: 0.1, block_unseen_share_hero: 0.2, block_unseen_share_monster: 0.3 }));
  it('gives campaign counts, late-vs-early, draw, unseen block', () => {
    const h = headline([{ run: run('r', '2026-10-01', { permadeath: { on: camp(3, 1), off: camp(8, 2) } }), rows }]);
    expect(h.campaign!.on).toMatchObject({ k: 3, n: 4 });
    expect(h.campaign!.off).toMatchObject({ k: 8, n: 10 });
    expect(h.lateVsEarly!.hero).toBeCloseTo(0.4); expect(h.lateVsEarly!.monster).toBeCloseTo(0.6);
    expect(h.draw).toBeCloseTo(0.1);
    expect(h.unseenBlock!.hero).toBeCloseTo(0.2); expect(h.unseenBlock!.monster).toBeCloseTo(0.3);
  });
  it('is null where data is missing', () => {
    const h = headline([{ run: run('r', '2026-10-01'), rows: [] }]);
    expect(h).toEqual({ campaign: null, lateVsEarly: null, draw: null, unseenBlock: null });
  });
});

describe('charts', () => {
  const xs = Array.from({ length: 50 }, (_, i) => i);
  const ys = xs.map((i) => (i % 2) * 0.5);
  const svg = lineChart([{ key: 'hero', label: 'hero', ys }, { key: 'monster', label: 'monster', ys }], xs, 1, { w: 300, h: 150 });
  it('sizes viewBox to pixels, has raw and smoothed lines, no legend inside', () => {
    expect(svg).toContain('viewBox="0 0 300 150"');
    expect((svg.match(/<polyline/g) ?? []).length).toBe(4);
    expect(svg).toContain('class="ln raw hero"');
    expect(svg).not.toMatch(/<text[^>]*class="lbl/);
    expect(svg).toContain('data-tip');
  });
  it('legend is HTML with line-style swatches', () => {
    const l = legendHtml([{ key: 'hero', label: 'hero' }, { key: 'monster', label: 'monster' }]);
    expect(l).toMatch(/class="lg hero"/);
    expect(l).toContain('monster');
  });
  it('tooltip finds the nearest generation', () => {
    expect(nearestIndex(0, 50, 30, 294)).toBe(0);
    expect(nearestIndex(1000, 50, 30, 294)).toBe(49);
    const t = tipText({ xs: [5, 6], s: [{ l: 'hero', ys: [0.5, 0.25] }], pct: true }, 1);
    expect(t).toContain('gen 6'); expect(t).toContain('hero'); expect(t).toContain('25%');
  });
  it('bar labels are two-line: rate above, counts under zone', () => {
    const b = barChart([{ label: 'Zone 1', k: 4, n: 5, rate: 0.8 }, { label: 'Zone 2', k: 5, n: 6, rate: 5 / 6 }], { w: 240, h: 140 });
    expect(b).toContain('80%'); expect(b).toContain('4/5'); expect(b).toContain('data-tip');
  });
});
