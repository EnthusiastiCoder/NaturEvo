// Assumed module: src/runs-index.ts (see v2/tests-index-viewer-landing.md)
import { describe, it, expect, vi } from 'vitest';
import { parseTrainingCsv, summarizeCampaign, buildIndex, loadIndex } from '../src/runs-index';

const CSV = 'gen,avg_ticks,best_hero,best_monster,mean_hero,mean_monster,hof_wr_hero,hof_wr_monster,rnd_wr_hero,rnd_wr_monster,hero_attack,hero_block,hero_dodge,monster_attack,monster_block,monster_dodge\n' +
  '0,1800,0.5,0.25,0.1,0.05,0,0,0,0,0.5,0.25,0.25,0.5,0.25,0.25\n' +
  '1,1700,2.5,1.5,0.4,0.2,0.4,0.6,0.8,0.2,0.2,0.5,0.3,0.3,0.5,0.2\n';

const camp = (outcome: string, results: [number, string][]) => JSON.stringify({
  map: {}, path: [], outcome,
  encounters: results.map(([zone, result], i) => ({ kind: 'elite', zone, tile: [0, i], result, fight: `fight_${i}.json` })),
});
const files = (m: Record<string, string>) => ({ list: () => Object.keys(m), read: async (p: string) => m[p] });

describe('parseTrainingCsv', () => {
  it('parses rows to numbers keyed by column', () => {
    const rows = parseTrainingCsv(CSV);
    expect(rows).toHaveLength(2);
    expect(rows[1]).toMatchObject({ gen: 1, avg_ticks: 1700, best_hero: 2.5, hof_wr_monster: 0.6, monster_dodge: 0.2 });
  });
  it('empty or header-only text gives []', () => {
    expect(parseTrainingCsv('')).toEqual([]);
    expect(parseTrainingCsv(CSV.split('\n')[0] + '\n')).toEqual([]);
  });
});

describe('summarizeCampaign', () => {
  it('counts wins, losses, encounters and per-zone', () => {
    const s = summarizeCampaign(camp('Died', [[1, 'HeroWin'], [1, 'MonsterWin'], [2, 'HeroWin'], [2, 'HeroWin']]))!;
    expect(s.outcome).toBe('Died');
    expect(s).toMatchObject({ wins: 3, losses: 1, encounters: 4 });
    expect(s.perZone).toEqual({ 1: { wins: 1, losses: 1 }, 2: { wins: 2, losses: 0 } });
  });
  it('invalid JSON returns null', () => {
    expect(summarizeCampaign('{nope')).toBeNull();
  });
});

describe('buildIndex', () => {
  it('derives generations, permadeath on/off, seed and n/a fields', async () => {
    const idx = await buildIndex(files({
      'main/training.csv': CSV,
      'main/campaign_on.json': camp('Died', [[1, 'MonsterWin']]),
      'main/campaign_off.json': camp('Cleared', [[1, 'HeroWin']]),
      'main/champion_hero_gen_9.json': '{}',
      'main/champion_hero_gen_29.json': '{}',
      'cd/s1/training.csv': CSV,
      'cd/s1/campaign_on.json': camp('Cleared', [[1, 'HeroWin']]),
    }), { mtime: () => '2026-10-01' });
    expect(idx.runs.map(r => r.name).sort()).toEqual(['cd/s1', 'main']);
    const main = idx.runs.find(r => r.name === 'main')!;
    expect(main.generations).toBe(2);
    expect(main.permadeath.on?.outcome).toBe('Died');
    expect(main.permadeath.off?.outcome).toBe('Cleared');
    expect(main.date).toBe('2026-10-01');
    expect(main.seed).toBe('n/a');
    expect(main.files).toContain('champion_hero_gen_29.json');
    const s1 = idx.runs.find(r => r.name === 'cd/s1')!;
    expect(s1.permadeath.off).toBeUndefined();
    expect(s1.seed).toBe('1');
  });
  it('skips junk folders and files without throwing', async () => {
    const idx = await buildIndex(files({
      '_vv/x.bin': 'zz', 'final/readme.txt': 'hi', 'tmp60/a.txt': 'a', 'notes.txt': 'n', 'py/a.py': 'x',
      'bad/training.csv': '\u0000\u0001garbage', 'bad/campaign_on.json': '{nope',
    }), { mtime: () => '2026-10-01' });
    const names = idx.runs.map(r => r.name);
    for (const junk of ['_vv', 'final', 'tmp60', 'py', 'notes.txt']) expect(names).not.toContain(junk);
    for (const r of idx.runs) expect(r.permadeath.on).toBeUndefined();
  });
  it('missing mtime gives n/a date', async () => {
    const idx = await buildIndex(files({ 'a/training.csv': CSV }), { mtime: () => undefined });
    expect(idx.runs[0].date).toBe('n/a');
  });
});

describe('loadIndex fallback chain', () => {
  const ok = (body: unknown) => ({ ok: true, json: async () => body }) as Response;
  const bad = () => ({ ok: false, status: 404, json: async () => ({}) }) as Response;
  const idx = { runs: [{ name: 'main' }] };

  it('uses /api/runs first', async () => {
    const f = vi.fn(async (u: string) => (u === '/api/runs' ? ok(idx) : bad()));
    expect(await loadIndex(f as any)).toEqual({ runs: [{ name: 'main' }] });
    expect(f).toHaveBeenCalledTimes(1);
  });
  it('falls back to /runs/index.json', async () => {
    const f = vi.fn(async (u: string) => (u === '/runs/index.json' ? ok(idx) : bad()));
    expect((await loadIndex(f as any)).runs).toHaveLength(1);
    expect(f.mock.calls.map(c => c[0])).toEqual(['/api/runs', '/runs/index.json']);
  });
  it('returns empty index with error reason when both fail or throw', async () => {
    const f = vi.fn(async () => { throw new Error('offline'); });
    const r = await loadIndex(f as any);
    expect(r.runs).toEqual([]);
    expect(typeof r.error).toBe('string');
    expect(r.error!.length).toBeGreaterThan(0);
  });
});
