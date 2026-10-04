// Landing view: run picker plus an at-a-glance results panel. View-model and HTML are pure; mountLanding does the DOM and fetching.
import { type RunIndex, type RunInfo, type Row, parseTrainingCsv } from './runs-index';
import { campaignWinRate, learningCurves, rateOf } from './aggregate';
import { type Size, barChart, legendHtml, lineChart, nearestIndex, pct, tipText, type Tip } from './charts';
import { headline } from './headline';
import { defaultRun, formatDate, groupRuns, hasCampaign, humanName } from './run-labels';

// The viewer reads v2/runs; the sim runs from v2/sim, so the commands write one level up.
export const EMPTY_COMMANDS: string[] = [
  'cargo run --release -- train --out ../runs/main --gens 200 --seed 1',
  'cargo run --release -- campaign --champions ../runs/main --out ../runs/main/campaign_on.json --fights-dir ../runs/main/fights_on --seed 1',
  'cargo run --release -- campaign --champions ../runs/main --out ../runs/main/campaign_off.json --fights-dir ../runs/main/fights_off --permadeath off --seed 1',
];

export type LandingRow = {
  name: string; label: string; seed: string; date: string; dateLabel: string; generations: number;
  permadeathOn: string; permadeathOff: string; onRate: string; offRate: string; keyResult: string;
  openHref: string; canOpen: boolean; hasCampaign: boolean;
};
export type LandingVM = { empty: boolean; rows: LandingRow[] };

const fightGens = (r: RunInfo) => r.files.map((f) => /^fight_gen_(\d+)\.json$/.exec(f)).filter((m): m is RegExpExecArray => !!m).map((m) => Number(m[1])).sort((a, b) => a - b);
const fightUrl = (r: RunInfo, g: number) => `/runs/${r.name}/fight_gen_${g}.json`;

/** Campaign runs open the jungle replay; training-only runs open the early-vs-late duel. */
function openFor(r: RunInfo): { href: string; ok: boolean } {
  if (hasCampaign(r)) return { href: `?run=${encodeURIComponent(r.name)}`, ok: true };
  const g = fightGens(r);
  if (g.length === 0) return { href: '', ok: false };
  const urls = g.length > 1 ? [g[0], g[g.length - 1]] : [g[0]];
  return { href: '?' + urls.map((x) => `fight=${encodeURIComponent(fightUrl(r, x))}`).join('&'), ok: true };
}

const rate = (s?: { wins: number; losses: number }) => (s ? pct(rateOf(s.wins, s.wins + s.losses).rate) : 'n/a');

export const landingViewModel = (index: RunIndex): LandingVM => ({
  empty: index.runs.length === 0,
  rows: index.runs.map((r) => {
    const o = openFor(r);
    return {
      name: r.name, label: humanName(r.name), seed: r.seed, date: r.date, dateLabel: formatDate(r.date), generations: r.generations,
      permadeathOn: r.permadeath.on?.outcome ?? 'n/a', permadeathOff: r.permadeath.off?.outcome ?? 'n/a',
      onRate: rate(r.permadeath.on), offRate: rate(r.permadeath.off),
      keyResult: r.keyResult, openHref: o.href, canOpen: o.ok, hasCampaign: hasCampaign(r),
    };
  }),
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
