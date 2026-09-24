// Landing view: run picker plus aggregate results. View-model and HTML are pure; mountLanding does the DOM and fetching.
import { type RunIndex, type RunInfo, type Row, parseTrainingCsv } from './runs-index';
import { campaignWinRate, learningCurves, poolRates, rateOf } from './aggregate';

export const EMPTY_COMMANDS: string[] = [
  'cargo run --release -- train --out runs/main --gens 200 --seed 1',
  'cargo run --release -- campaign --champions runs/main --out runs/main/campaign_on.json --fights-dir runs/main/fights_on --seed 1',
];

export type LandingRow = {
  name: string; seed: string; date: string; generations: number;
  permadeathOn: string; permadeathOff: string; keyResult: string; openHref: string;
};
export type LandingVM = { empty: boolean; rows: LandingRow[] };

export const landingViewModel = (index: RunIndex): LandingVM => ({
  empty: index.runs.length === 0,
  rows: index.runs.map((r) => ({
    name: r.name, seed: r.seed, date: r.date, generations: r.generations,
    permadeathOn: r.permadeath.on?.outcome ?? 'n/a', permadeathOff: r.permadeath.off?.outcome ?? 'n/a',
    keyResult: r.keyResult, openHref: `?run=${encodeURIComponent(r.name)}`,
  })),
});

const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const icon = (d: string) => `<svg viewBox="0 0 24 24" aria-hidden="true">${d}</svg>`;
const ICON = {
  play: '<path d="m6 4 14 8-14 8z"/>',
  open: '<path d="M5 12h14M13 6l6 6-6 6"/>',
  info: '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>',
  layers: '<path d="m12 3 9 5-9 5-9-5z"/><path d="m3 13 9 5 9-5"/>',
};
export const infoButton = (text: string) =>
  `<button type="button" class="info-btn" aria-label="About this chart" aria-expanded="false" title="${esc(text)}">${icon(ICON.info)}</button>`;

const demoButton = `<button type="button" data-action="demo" title="Open the built-in demo fight (not a real run)">Demo${icon(ICON.play)}</button>`;
