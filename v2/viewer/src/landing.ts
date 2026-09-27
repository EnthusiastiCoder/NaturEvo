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

const demoButton = `<button type="button" data-action="demo" title="Open the built-in demo fight (not a real run)">Demo${icon(ICON.play)}</button>`;

function rowHtml(r: LandingRow, selected: boolean): string {
  const chips = r.hasCampaign
    ? `<i class="chip win" title="Campaign win rate with permadeath on">on ${esc(r.onRate)}</i><i class="chip win" title="Campaign win rate with permadeath off">off ${esc(r.offRate)}</i>`
    : '<i class="chip" title="Trained, but no campaign has been run yet">training only</i>';
  const open = r.canOpen ? `<a class="open" href="${esc(r.openHref)}" tabindex="-1" title="${r.hasCampaign ? 'Open the jungle campaign replay' : 'Open the early-versus-late duel'}">Open${icon(ICON.open)}</a>` : '<span class="open off" title="No replay files in this run">no replay</span>';
  const meta = [r.dateLabel, r.generations ? `${r.generations} gen` : ''].filter(Boolean).join(' · ');
  return `<li><div class="run-row" role="option" tabindex="${selected ? 0 : -1}" aria-selected="${selected}" data-run="${esc(r.name)}" data-open="${esc(r.openHref)}" title="${esc(r.name)}">
<b class="nm">${esc(r.label)}</b>${open}<span class="meta">${esc(meta)}</span><span class="chips">${chips}</span></div></li>`;
}

export function renderLandingHtml(vm: LandingVM, selectedName?: string): string {
  if (vm.empty) {
    return `<section class="landing-empty"><h2>No runs yet</h2>
<p>Train a population, then walk the jungle with its champions. Run these from the <code>v2/sim</code> folder; they write into <code>v2/runs/</code>, which is where this viewer looks.</p>
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

const card = (title: string, info: string, body: string) =>
  `<article class="card"><header><h3>${esc(title)}</h3>${infoButton(info)}</header><p class="info-text" hidden>${esc(info)}</p><div class="body">${body}</div></article>`;

export type RunData = { run: RunInfo; rows: Row[] };

export function renderResultsHtml(data: RunData[], all: boolean): string {
  const sel = data[0];
  const camp = (side: 'on' | 'off') => poolRates(data.map((x) => { const s = x.run.permadeath[side]; return s ? { k: s.wins, n: s.wins + s.losses } : { k: 0, n: 0 }; }));
  const wr = all ? { on: camp('on'), off: camp('off') } : campaignWinRate({ on: sel.run.permadeath.on, off: sel.run.permadeath.off });
  const winBars = wr.on.n + wr.off.n === 0 ? na('No campaign files for this selection')
    : barChart([{ label: 'permadeath on', ...wr.on }, { label: 'permadeath off', ...wr.off }]);
  const zones = new Map<string, { k: number; n: number }>();
  for (const d of data) for (const side of ['on', 'off'] as const) for (const [z, v] of Object.entries(d.run.permadeath[side]?.perZone ?? {})) {
    const a = zones.get(z) ?? { k: 0, n: 0 }; a.k += v.wins; a.n += v.wins + v.losses; zones.set(z, a);
  }
  const zr = [...zones].sort((a, b) => Number(a[0]) - Number(b[0])).map(([z, a]) => ({ label: `zone ${z}`, ...rateOf(a.k, a.n) }));
  const rows = all ? [] : sel.rows;
  const c = learningCurves(rows);
  const noCsv = (body: () => string) => (all ? na('Pick a single run: curves are per training run') : rows.length ? body() : na('No training.csv for this run'));
  const has = (col: string) => rows.some((r) => r[col] !== undefined);
  const col = (k: string) => rows.map((r) => r[k] ?? 0);
  const fit = noCsv(() => lineChart([{ cls: 'hero', label: 'hero', ys: c.heroFitness }, { cls: 'monster', label: 'monster', ys: c.monsterFitness }], c.gens, Math.max(1, ...c.heroFitness, ...c.monsterFitness)));
  const vs = noCsv(() => lineChart([{ cls: 'hero', label: 'hero win', ys: c.heroWinVsEarlier }, { cls: 'monster', label: 'monster win', ys: c.monsterWinVsEarlier }, ...(c.drawRate ? [{ cls: 'draw', label: 'draw', ys: c.drawRate }] : [])], c.gens, 1));
  const seen = noCsv(() => has('seen_share_hero')
    ? lineChart([{ cls: 'hero', label: 'hero sees foe', ys: col('seen_share_hero') }, { cls: 'monster', label: 'monster sees foe', ys: col('seen_share_monster') }], c.gens, 1)
    : na('This run predates the vision stats in training.csv'));
  const stat = noCsv(() => has('statuses_per_fight_hero')
    ? lineChart([{ cls: 'hero', label: 'on hero', ys: col('statuses_per_fight_hero') }, { cls: 'monster', label: 'on monster', ys: col('statuses_per_fight_monster') }], c.gens, Math.max(1, ...col('statuses_per_fight_hero'), ...col('statuses_per_fight_monster')))
    : na('This run predates status stats in training.csv'));
  const gens = sel.run.files.map((f) => /^fight_gen_(\d+)\.json$/.exec(f)).filter((m): m is RegExpExecArray => !!m).map((m) => Number(m[1])).sort((a, b) => a - b);
  const link = (g: number, label: string) => `<a href="?fight=${encodeURIComponent(`/runs/${sel.run.name}/fight_gen_${g}.json`)}" title="Replay the ${label} fight (generation ${g})">${label} (gen ${g})</a>`;
  const duel = !all && gens.length > 1 ? `<span class="duels">Duel: ${link(gens[0], 'early')} ${link(gens[gens.length - 1], 'late')}</span>` : '';
  const title = all ? `All ${data.length} runs` : sel.run.name;
  return `<div class="results-head"><h2>${esc(title)}</h2>${duel}</div><div class="grid">
${card('Campaign win rate', 'Share of jungle encounters the hero won, with permadeath on versus off. Bars start at zero; labels show wins of encounters.', winBars)}
${card('Win rate by zone', 'Hero win share per jungle zone, pooled over campaign files. Later zones are tougher.', zr.length ? barChart(zr) : na('No encounters recorded'))}
${card('Best fitness', 'Best fitness per generation for each side. Rising lines mean both sides keep learning.', fit)}
${card('Win vs earlier champions', 'Win share against earlier hall-of-fame champions; draw is what is left. Rates run from 0 to 1.', vs)}
${card('Sees the opponent', 'Share of ticks where each side had the opponent in its vision cone, per generation.', seen)}
${card('Statuses per fight', 'Average status effects (stun, bleed and so on) landed per fight, per generation.', stat)}</div>`;
}

// ---- DOM controller ----
export async function mountLanding(el: HTMLElement, index: RunIndex, onDemo: () => void): Promise<void> {
  el.innerHTML = renderLandingHtml(landingViewModel(index));
  el.querySelector('[data-action="demo"]')?.addEventListener('click', onDemo);
  const results = el.querySelector<HTMLElement>('#landing-results');
  if (!results) return;
  const text = async (u: string) => { try { const r = await fetch(u); return r.ok ? await r.text() : ''; } catch { return ''; } };
  const cache = new Map<string, Row[]>();
  const rowsOf = async (r: RunInfo) => {
    if (!cache.has(r.name)) cache.set(r.name, parseTrainingCsv(await text(`/runs/${r.name}/training.csv`)));
    return cache.get(r.name)!;
  };
  let selected = index.runs[0];
  let all = false;
  const paint = async () => {
    const data = all ? index.runs.map((run) => ({ run, rows: [] as Row[] })) : [{ run: selected, rows: await rowsOf(selected) }];
    results.innerHTML = renderResultsHtml(data, all);
    results.querySelectorAll<HTMLButtonElement>('.info-btn').forEach((b) => b.addEventListener('click', () => {
      const p = b.closest('.card')!.querySelector<HTMLElement>('.info-text')!;
      p.hidden = !p.hidden; b.setAttribute('aria-expanded', String(!p.hidden));
    }));
    el.querySelectorAll<HTMLElement>('.run-row').forEach((a) => a.parentElement!.classList.toggle('sel', !all && a.dataset.run === selected.name));
  };
  el.querySelectorAll<HTMLAnchorElement>('.run-row').forEach((a) => {
    // Hover or keyboard focus previews the run in the results panel; click or Enter opens it in the replay.
    const preview = () => { selected = index.runs.find((r) => r.name === a.dataset.run)!; all = false; el.querySelector('[data-action="all-runs"]')?.setAttribute('aria-pressed', 'false'); void paint(); };
    a.addEventListener('focus', preview); a.addEventListener('mouseenter', preview);
  });
  el.querySelector('[data-action="all-runs"]')?.addEventListener('click', (e) => {
    all = !all; (e.currentTarget as HTMLElement).setAttribute('aria-pressed', String(all)); void paint();
  });
  await paint();
}
