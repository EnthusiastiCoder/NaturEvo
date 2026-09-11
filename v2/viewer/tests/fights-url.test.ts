import { describe, it, expect } from 'vitest';
import { parseRun, resolveFightUrl, resolveFights, mergeDuels, type Fight } from '../src/data';

const fightJson = { tick_hz: 30, seed: 1, monster_index: 0, arena: { w: 48, h: 32 },
  frames: [{ pos: [[1, 1], [2, 2]], facing: [0, 0], hp: [100, 100], stamina: [100, 100], action: ['none', 'none'], state: ['Idle', 'Idle'] }] };
const runJson = (fight: unknown) => ({ map: { w: 1, h: 1, tiles: ['.'], zones: ['0'], anchors: [] }, path: [[0, 0]], encounters: [{ kind: 'grass', tile: [0, 0], fight }] });

describe('relative fight urls', () => {
  it('resolves against the run file url, not the page origin', () => {
    expect(resolveFightUrl('fight_0.json', 'http://h:5174/@fs/C:/x/runs/campaign_on.json')).toBe('http://h:5174/@fs/C:/x/runs/fight_0.json');
    expect(resolveFightUrl('/abs/f.json', 'http://h/a/b.json')).toBe('http://h/abs/f.json');
  });
  it('fetches the resolved url and reports failures instead of dropping silently', async () => {
    const run = parseRun(runJson('fight_0.json'));
    const seen: string[] = [];
    const ok = async (u: string) => { seen.push(u); return { ok: true, json: async () => fightJson } as Response; };
    expect(await resolveFights(run, 'http://h/runs/r.json', ok)).toEqual([]);
    expect(seen).toEqual(['http://h/runs/fight_0.json']);
    expect(run.encounters[0].fight?.frames.length).toBe(1);
    const run2 = parseRun(runJson('fight_0.json'));
    const bad = async () => ({ ok: true, json: async () => { throw new Error('Unexpected token <'); } }) as unknown as Response;
    const errs = await resolveFights(run2, 'http://h/runs/r.json', bad);
    expect(errs.length).toBe(1);
    expect(errs[0]).toContain('fight_0.json');
  });
  it('reports http errors', async () => {
    const run = parseRun(runJson('fight_0.json'));
    const errs = await resolveFights(run, 'http://h/r.json', async () => ({ ok: false, status: 404 }) as Response);
    expect(errs[0]).toContain('404');
  });
});
