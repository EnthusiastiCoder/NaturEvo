// The few numbers that answer "how is this run doing" at a glance.
import { type Rate, poolRates, tailMean } from './aggregate';
import type { Row, RunInfo } from './runs-index';

export type Headline = {
  campaign: { on: Rate; off: Rate } | null;
  lateVsEarly: { hero: number; monster: number } | null;
  draw: number | null;
  unseenBlock: { hero: number; monster: number } | null;
};
const TAIL = 10; // last generations averaged
