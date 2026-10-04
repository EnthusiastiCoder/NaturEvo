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
