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
