// Assumed module: src/landing.ts exports landingViewModel({runs, error?}) and renderLandingHtml(vm).
// Run shape: {name, seed, date, generations, fightGens: number[]}.
// vm: {empty: boolean, rows: [{name, openHref, ...}]}
import { describe, it, expect } from 'vitest';
import { landingViewModel, renderLandingHtml } from '../src/landing';

const run = (name: string, fightGens: number[] = [0, 10, 200]) =>
  ({ name, seed: '7', date: '2026-10-24', generations: 200, fightGens }) as any;

describe('landing empty state', () => {
  it('says no runs yet and shows the train command, no demo', () => {
    const vm = landingViewModel({ runs: [] });
    expect(vm.empty).toBe(true);
    const html = renderLandingHtml(vm);
    expect(html).toMatch(/no runs yet/i);
    expect(html).toContain('cargo run --release -- train');
    expect(html).not.toMatch(/demo/i);
  });
  it('an index error still renders the empty state without a banner', () => {
    const html = renderLandingHtml(landingViewModel({ runs: [], error: 'offline' }));
    expect(html).toMatch(/no runs yet/i);
    expect(html).not.toMatch(/demo/i);
    expect(html).not.toMatch(/class="[^"]*banner/);
  });
});

describe('landing with runs', () => {
  const runs = [run('slice_s1'), run('slice s2', [0, 50])];
  it('lists every run and never mentions demo', () => {
    const vm: any = landingViewModel({ runs });
    expect(vm.empty).toBe(false);
    expect(vm.rows).toHaveLength(2);
    const html = renderLandingHtml(vm);
    expect(html).toContain('slice_s1');
    expect(html).not.toMatch(/demo/i);
  });
  it('openHref points to the latest fight generation with encoded name', () => {
    const vm: any = landingViewModel({ runs });
    expect(vm.rows[0].openHref).toBe('#/fight/slice_s1/200');
    expect(vm.rows[1].openHref).toBe(`#/fight/${encodeURIComponent('slice s2')}/50`);
  });
  it('has exactly one primary Watch fight action', () => {
    const html = renderLandingHtml(landingViewModel({ runs }));
    const n = (html.match(/data-action="watch"/g) ?? []).length;
    expect(n).toBe(1);
    expect(html).toContain('Watch fight');
  });
});
