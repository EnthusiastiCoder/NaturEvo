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
