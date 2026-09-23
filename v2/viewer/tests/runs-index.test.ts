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
