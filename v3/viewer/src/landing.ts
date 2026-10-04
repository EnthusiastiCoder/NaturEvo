// Landing view: run picker plus an at-a-glance results panel. View-model and HTML are pure; mountLanding does the DOM and fetching.
import { type RunIndex, type RunInfo, type Row, parseTrainingCsv } from './runs-index';
import { campaignWinRate, learningCurves, rateOf } from './aggregate';
import { type Size, barChart, legendHtml, lineChart, nearestIndex, pct, tipText, type Tip } from './charts';
import { headline } from './headline';
import { defaultRun, formatDate, groupRuns, hasCampaign, humanName } from './run-labels';
