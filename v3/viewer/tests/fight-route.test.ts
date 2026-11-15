import { describe, it, expect } from 'vitest';
import { fightRoute } from '../src/fight-route';
import { buildIndex } from '../src/runs-index';

describe('fightRoute', () => {
  it('opens a generation fight', () => {
    expect(fightRoute('#/fight/style_press/999')).toEqual({ url: '/runs/style_press/fight_gen_999.json', title: 'style_press gen 999' });
  });
  it('opens a featured long fight', () => {
    expect(fightRoute('#/featured/featured_press_vs_duelist')).toEqual({ url: '/runs/featured_press_vs_duelist.json', title: 'featured press vs duelist' });
  });
  it('rejects paths that are not plain featured names', () => {
    expect(fightRoute('#/featured/../secret')).toBeNull();
    expect(fightRoute('#/featured/other_file')).toBeNull();
    expect(fightRoute('')).toBeNull();
  });
});

describe('index lists featured fights', () => {
  it('collects featured_*.json files at the runs root', async () => {
    const files = ['featured_a_vs_b.json', 'featured_c_vs_d.json', 'eval.md', 'r1/run.json'];
    const idx = await buildIndex({ list: () => files, read: async () => '{"name":"r1","generations":1}' }, { mtime: () => undefined });
    expect(idx.featured).toEqual(['featured_a_vs_b', 'featured_c_vs_d']);
    expect(idx.runs.length).toBe(1);
  });
});
