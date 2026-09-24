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

export function renderLandingHtml(vm: LandingVM): string {
  if (vm.empty) {
    return `<section class="landing-empty"><h2>No runs yet</h2>
<p>Train a population, then walk the jungle with its champions. Run these in <code>v2/sim</code>:</p>
<ol class="cmds">${EMPTY_COMMANDS.map((c) => `<li><code>${esc(c)}</code></li>`).join('')}</ol>
<p class="muted">Results appear here when the files exist. Meanwhile you can look at the built-in demo.</p>
<div class="actions">${demoButton}</div></section>`;
  }
  const rows = vm.rows.map((r) => `<li><a class="run-row" href="${esc(r.openHref)}" data-run="${esc(r.name)}" title="Open ${esc(r.name)} in the replay">
<b class="nm">${esc(r.name)}</b><span title="Seed">seed ${esc(r.seed)}</span><span title="Last modified">${esc(r.date)}</span><span title="Generations trained">${r.generations} gen</span>
<span title="Campaign outcome with permadeath on / off">on ${esc(r.permadeathOn)} / off ${esc(r.permadeathOff)}</span>${icon(ICON.open)}</a></li>`).join('');
  return `<section class="picker"><div class="head"><h2>Runs</h2>
<button type="button" data-action="all-runs" aria-pressed="false" title="Pool campaign results across every run">${icon(ICON.layers)}All runs</button>${demoButton}</div>
<ul class="runs">${rows}</ul></section>
<section class="results" id="landing-results" aria-live="polite"></section>`;
}

// ---- charts (inline SVG, colours only via CSS classes and tokens) ----
const W = 240, H = 120, PAD = { l: 28, r: 8, t: 8, b: 16 };
const pct = (r: number | null) => (r === null ? 'n/a' : `${Math.round(r * 100)}%`);
const na = (why: string) => `<div class="na" title="${esc(why)}">n/a<small>${esc(why)}</small></div>`;

function lineChart(series: { cls: string; label: string; ys: number[] }[], xs: number[], yMax: number): string {
  const px = (i: number) => PAD.l + (xs.length > 1 ? (i / (xs.length - 1)) * (W - PAD.l - PAD.r) : 0);
  const py = (v: number) => H - PAD.b - (Math.max(0, v) / yMax) * (H - PAD.t - PAD.b);
  const lines = series.map((s) => `<polyline class="ln ${s.cls}" points="${s.ys.map((v, i) => `${px(i).toFixed(1)},${py(v).toFixed(1)}`).join(' ')}"><title>${esc(s.label)}</title></polyline>`).join('');
  const labels = series.map((s, i) => `<text class="lbl ${s.cls}" x="${W - PAD.r}" y="${PAD.t + 8 + i * 10}" text-anchor="end">${esc(s.label)}</text>`).join('');
  return `<svg class="chart" viewBox="0 0 ${W} ${H}" role="img"><line class="axis" x1="${PAD.l}" y1="${py(0)}" x2="${W - PAD.r}" y2="${py(0)}"/>
<text class="tick" x="${PAD.l - 3}" y="${py(0) + 3}" text-anchor="end">0</text><text class="tick" x="${PAD.l - 3}" y="${py(yMax) + 3}" text-anchor="end">${+yMax.toFixed(1)}</text>
<text class="tick" x="${PAD.l}" y="${H - 3}">gen ${xs[0]}</text><text class="tick" x="${W - PAD.r}" y="${H - 3}" text-anchor="end">${xs[xs.length - 1]}</text>${lines}${labels}</svg>`;
}

function barChart(bars: { label: string; k: number; n: number; rate: number | null }[]): string {
  const bw = (W - PAD.l - PAD.r) / Math.max(1, bars.length), base = H - PAD.b - 4, top = PAD.t + 10;
  const g = bars.map((b, i) => {
    const h = b.rate === null ? 0 : b.rate * (base - top), x = PAD.l + i * bw + bw * 0.2;
    return `<g><title>${esc(b.label)}: ${b.k} of ${b.n}</title><rect class="bar" x="${x}" y="${base - h}" width="${bw * 0.6}" height="${h}"/>
<text class="lbl" x="${x + bw * 0.3}" y="${base - h - 3}" text-anchor="middle">${pct(b.rate)} (${b.k}/${b.n})</text>
<text class="tick" x="${x + bw * 0.3}" y="${H - 3}" text-anchor="middle">${esc(b.label)}</text></g>`;
  }).join('');
  return `<svg class="chart" viewBox="0 0 ${W} ${H}" role="img"><line class="axis" x1="${PAD.l}" y1="${base}" x2="${W - PAD.r}" y2="${base}"/>
<text class="tick" x="${PAD.l - 3}" y="${base + 3}" text-anchor="end">0</text><text class="tick" x="${PAD.l - 3}" y="${top + 3}" text-anchor="end">100%</text>${g}</svg>`;
}
