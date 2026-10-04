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
