// At-a-glance landing: labels, grouping, headline numbers, smoothing, charts, empty state.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { humanName, isScratch, groupRuns, defaultRun, formatDate } from '../src/run-labels';
import { smooth, tailMean } from '../src/aggregate';
import { headline } from '../src/headline';
import { lineChart, barChart, legendHtml, nearestIndex, tipText } from '../src/charts';
import { landingViewModel, renderLandingHtml, renderResultsHtml, EMPTY_COMMANDS } from '../src/landing';

const camp = (wins: number, losses: number) => ({ outcome: 'Died', wins, losses, encounters: wins + losses, perZone: { '1': { wins, losses } } });
const run = (name: string, date: string, extra: any = {}) => ({
  name, seed: 'n/a', date, generations: 200, permadeath: {}, keyResult: 'n/a',
  files: ['training.csv', 'fight_gen_9.json', 'fight_gen_199.json'], ...extra,
}) as any;
