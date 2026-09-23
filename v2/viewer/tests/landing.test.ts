// Assumed module: src/landing.ts (see v2/tests-index-viewer-landing.md)
import { describe, it, expect } from 'vitest';
import { landingViewModel, renderLandingHtml, EMPTY_COMMANDS } from '../src/landing';

const run = (name: string, extra = {}) => ({
  name, seed: '7', date: '2026-10-01', generations: 30,
  permadeath: { on: { outcome: 'Died', wins: 1, losses: 1, encounters: 2, perZone: {} }, off: undefined },
  keyResult: 'Died', files: [], ...extra,
}) as any;

describe('landing empty state', () => {
  it('lists exact CLI commands and is not an error banner', () => {
    const vm = landingViewModel({ runs: [] });
    expect(vm.empty).toBe(true);
    expect(EMPTY_COMMANDS).toEqual(expect.arrayContaining([
      expect.stringContaining('cargo run --release -- train'),
      expect.stringContaining('cargo run --release -- campaign'),
    ]));
    const html = renderLandingHtml(vm);
    for (const c of EMPTY_COMMANDS) expect(html).toContain(c);
    expect(html).not.toMatch(/banner|error|red/i);
  });
  it('index error still shows empty state, not a banner', () => {
    const html = renderLandingHtml(landingViewModel({ runs: [], error: 'offline' }));
    for (const c of EMPTY_COMMANDS) expect(html).toContain(c);
    expect(html).not.toMatch(/class="[^"]*banner/);
  });
});

describe('landing picker', () => {
  const vm = landingViewModel({ runs: [run('main'), run('cd/s1', { seed: 'n/a' })] });
  it('rows carry name, seed, date, gens, permadeath on/off and key result', () => {
    expect(vm.empty).toBe(false);
    expect(vm.rows).toHaveLength(2);
    expect(vm.rows[0]).toMatchObject({ name: 'main', seed: '7', date: '2026-10-01', generations: 30, permadeathOn: 'Died', permadeathOff: 'n/a', keyResult: 'Died' });
    expect(vm.rows[1].seed).toBe('n/a');
  });
  it('Open link targets ?run=<encoded name>', () => {
    expect(vm.rows[0].openHref).toBe('?run=main');
    expect(vm.rows[1].openHref).toBe('?run=cd%2Fs1');
  });
  it('has a Demo button and an All runs toggle', () => {
    const html = renderLandingHtml(vm);
    expect(html).toMatch(/<button[^>]*data-action="demo"[^>]*>[^<]*Demo/);
    expect(html).toContain('data-action="all-runs"');
    expect(html).toContain('href="?run=main"');
  });
  it('html has no hard-coded colours', () => {
    expect(renderLandingHtml(vm)).not.toMatch(/#[0-9a-fA-F]{3,8}\b|rgb\(/);
  });
});
