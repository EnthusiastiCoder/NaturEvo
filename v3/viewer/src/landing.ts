// Landing view: run picker plus aggregate results. View-model and HTML are pure; mountLanding does the DOM and fetching.
import { type RunIndex, type RunInfo, type Row, parseTrainingCsv } from './runs-index';
import { type EvalRow, aggregateEval, aggregateSeries, defenceShares } from './aggregate';
import { type Size, barChart, legendHtml, lineChart, nearestIndex, pct, tipText, type Tip } from './charts';
import { formatDate, humanName } from './run-labels';
import { DEFAULT_ACCENT, applyTheme, nextTheme, setAccent, themeIconPath, type Theme } from './theme';

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
  const groups = groupRunsFromVm(vm);
  const sel = selectedName ?? groups.campaign[0]?.name ?? groups.training[0]?.name ?? groups.scratch[0]?.name;
  const list = (rs: LandingRow[]) => `<ul>${rs.map((r) => rowHtml(r, r.name === sel)).join('')}</ul>`;
  const section = (title: string, rs: LandingRow[], hint = '') => (rs.length ? `<h3 class="grp">${esc(title)}${hint ? `<small>${esc(hint)}</small>` : ''}</h3>${list(rs)}` : '');
  const scratchOpen = groups.scratch.some((r) => r.name === sel) ? ' open' : '';
  const scratch = groups.scratch.length ? `<details class="scratch"${scratchOpen}><summary>Scratch runs (${groups.scratch.length})</summary>${list(groups.scratch)}</details>` : '';
  return `<section class="picker"><div class="head"><h2>Runs</h2>
<button type="button" data-action="all-runs" aria-pressed="false" title="Pool the campaign results of every run that has them">${icon(ICON.layers)}All runs</button>${demoButton}</div>
<div class="runs" role="listbox" aria-label="Runs">${section('With campaign results', groups.campaign, 'win rate: permadeath on / off')}${section('Training only', groups.training)}${scratch}</div></section>
<section class="results" id="landing-results" aria-live="polite"></section>`;
}

function groupRunsFromVm(vm: LandingVM): { campaign: LandingRow[]; training: LandingRow[]; scratch: LandingRow[] } {
  const stub = vm.rows.map((r) => ({ name: r.name, seed: r.seed, date: r.date, generations: r.generations, permadeath: r.hasCampaign ? { on: { outcome: '', wins: 0, losses: 0, encounters: 0, perZone: {} } } : {}, keyResult: r.keyResult, files: [] }) as RunInfo);
  const g = groupRuns(stub), by = new Map(vm.rows.map((r) => [r.name, r]));
  const m = (rs: RunInfo[]) => rs.map((r) => by.get(r.name)!);
  return { campaign: m(g.campaign), training: m(g.training), scratch: m(g.scratch) };
}

// ---- results ----
const na = (why: string) => `<div class="na">${esc(why)}</div>`;
export type RunData = { run: RunInfo; rows: Row[] };
const DEFAULT_SIZE: Size = { w: 240, h: 130 };

const card = (title: string, info: string, body: string, legend = '') =>
  `<article class="card"><header><h3>${esc(title)}</h3>${legend}${infoButton(info)}</header><p class="info-text" hidden>${esc(info)}</p><div class="body">${body}</div></article>`;
const tile = (label: string, info: string, big: string, sub: string) =>
  `<div class="tile" title="${esc(info)}"><span class="tl">${esc(label)}</span><span class="big">${big}</span><span class="sub">${esc(sub)}</span></div>`;
const duo = (a: string, b: string) => `<span class="hero">${esc(a)}</span><span class="sep">/</span><span class="monster">${esc(b)}</span>`;

export function renderResultsHtml(data: RunData[], all: boolean, opts: { total?: number; size?: Size } = {}): string {
  const size = opts.size ?? DEFAULT_SIZE;
  const sel = data[0];
  const hl = headline(data);
  const tiles: string[] = [];
  if (hl.campaign) {
    const f = (r: { k: number; n: number; rate: number | null }) => `${pct(r.rate)} <small>${r.k}/${r.n}</small>`;
    tiles.push(`<div class="tile wide" title="Share of jungle encounters the hero won"><span class="tl">Campaign win rate</span><span class="big">${hl.campaign.on.n ? `<span class="k">perma on</span> ${f(hl.campaign.on)}` : ''}${hl.campaign.off.n ? ` <span class="k">off</span> ${f(hl.campaign.off)}` : ''}</span><span class="sub">permadeath on / off, wins of encounters</span></div>`);
  } else if (!all) tiles.push(tile('Campaign win rate', 'No campaign files for this run', 'none yet', 'run the campaign command to fill this'));
  if (hl.lateVsEarly) tiles.push(tile('Late vs early champions', 'Win share of current champions against earlier hall-of-fame champions (mean of the last 10 generations)', duo(pct(hl.lateVsEarly.hero), pct(hl.lateVsEarly.monster)), 'hero / monster win share'));
  if (hl.draw !== null) tiles.push(tile('Draw rate', 'Share of duels that end without a winner (last 10 generations)', pct(hl.draw), 'duels with no winner'));
  if (hl.unseenBlock) tiles.push(tile('Blocks at an unseen foe', 'Share of blocks started while the opponent was outside the vision cone (last 10 generations). Low is good.', duo(pct(hl.unseenBlock.hero), pct(hl.unseenBlock.monster)), 'hero / monster, lower is smarter'));

  const cards: string[] = [];
  const zones = new Map<string, { k: number; n: number }>();
  for (const d of data) for (const side of ['on', 'off'] as const) for (const [z, v] of Object.entries(d.run.permadeath[side]?.perZone ?? {})) {
    const a = zones.get(z) ?? { k: 0, n: 0 }; a.k += v.wins; a.n += v.wins + v.losses; zones.set(z, a);
  }
  if (zones.size) cards.push(card('Win rate by zone', 'Hero win share per jungle zone, pooled over the campaign files. Later zones are tougher.',
    barChart([...zones].sort((a, b) => Number(a[0]) - Number(b[0])).map(([z, a]) => ({ label: `Z${z}`, ...rateOf(a.k, a.n) })), size)));
  const rows = all ? [] : sel.rows;
  if (rows.length) {
    const c = learningCurves(rows);
    const has = (col: string) => rows.some((r) => r[col] !== undefined);
    const col = (k: string) => rows.map((r) => r[k] ?? 0);
    const hm = (a: string, b: string) => legendHtml([{ key: 'hero', label: a }, { key: 'monster', label: b }]);
    cards.push(card('Best fitness', 'Best fitness per generation for each side. Rising lines mean both sides keep learning. Faint line is raw, bold is smoothed.',
      lineChart([{ key: 'hero', label: 'hero', ys: c.heroFitness }, { key: 'monster', label: 'monster', ys: c.monsterFitness }], c.gens, Math.max(1, ...c.heroFitness, ...c.monsterFitness), size), hm('hero', 'monster')));
    cards.push(card('Win vs earlier champions', 'Win share against earlier hall-of-fame champions; draw is what is left. Faint line is raw, bold is smoothed.',
      lineChart([{ key: 'hero', label: 'hero win', ys: c.heroWinVsEarlier }, { key: 'monster', label: 'monster win', ys: c.monsterWinVsEarlier }, ...(c.drawRate ? [{ key: 'draw' as const, label: 'draw', ys: c.drawRate }] : [])], c.gens, 1, size, { pct: true }),
      legendHtml([{ key: 'hero', label: 'hero' }, { key: 'monster', label: 'monster' }, { key: 'draw', label: 'draw' }])));
    if (has('seen_share_hero')) cards.push(card('Sees the opponent', 'Share of ticks where each side had the opponent in its vision cone, per generation.',
      lineChart([{ key: 'hero', label: 'hero sees foe', ys: col('seen_share_hero') }, { key: 'monster', label: 'monster sees foe', ys: col('seen_share_monster') }], c.gens, 1, size, { pct: true }), hm('hero', 'monster')));
    if (has('statuses_per_fight_hero')) cards.push(card('Statuses per fight', 'Average status effects (stun, bleed and so on) landed per fight, per generation. "on hero" means suffered by the hero.',
      lineChart([{ key: 'hero', label: 'on hero', ys: col('statuses_per_fight_hero') }, { key: 'monster', label: 'on monster', ys: col('statuses_per_fight_monster') }], c.gens, Math.max(1, ...col('statuses_per_fight_hero'), ...col('statuses_per_fight_monster')), size), hm('on hero', 'on monster')));
  }
  const gens = fightGens(sel.run);
  const link = (g: number, label: string) => `<a href="?fight=${encodeURIComponent(fightUrl(sel.run, g))}" title="Replay the ${label} fight (generation ${g})">${label} (gen ${g})</a>`;
  const duel = !all && gens.length > 1 ? `<span class="duels">Replay: ${link(gens[0], 'early')} ${link(gens[gens.length - 1], 'late')}</span>` : '';
  const title = all ? 'All runs' : humanName(sel.run.name);
  const sub = all ? `${data.length} of ${opts.total ?? data.length} runs have campaign data; only those are pooled` : [formatDate(sel.run.date), sel.run.generations ? `${sel.run.generations} generations` : ''].filter(Boolean).join(' · ');
  const body = cards.length ? `<div class="grid">${cards.join('')}</div>` : na('Nothing to chart for this selection yet');
  return `<div class="results-head"><h2>${esc(title)}</h2><span class="sub">${esc(sub)}</span>${duel}</div>${tiles.length ? `<div class="stats">${tiles.join('')}</div>` : ''}${body}`;
}

// ---- DOM controller ----
export async function mountLanding(el: HTMLElement, index: RunIndex, onDemo: () => void): Promise<void> {
  const start = defaultRun(index.runs);
  el.innerHTML = renderLandingHtml(landingViewModel(index), start?.name);
  el.querySelector('[data-action="demo"]')?.addEventListener('click', onDemo);
  const results = el.querySelector<HTMLElement>('#landing-results');
  if (!results || !start) return;
  const text = async (u: string) => { try { const r = await fetch(u); return r.ok ? await r.text() : ''; } catch { return ''; } };
  const cache = new Map<string, Row[]>();
  const rowsOf = async (r: RunInfo) => {
    if (!cache.has(r.name)) cache.set(r.name, parseTrainingCsv(await text(`/runs/${r.name}/training.csv`)));
    return cache.get(r.name)!;
  };
  let selected = start, all = false, size = DEFAULT_SIZE;
  const withCampaign = index.runs.filter(hasCampaign);
  let tip = results.querySelector<HTMLElement>('.tip');
  const paint = async () => {
    const data = all ? await Promise.all(withCampaign.map(async (run) => ({ run, rows: await rowsOf(run) }))) : [{ run: selected, rows: await rowsOf(selected) }];
    results.innerHTML = renderResultsHtml(data, all, { total: index.runs.length, size });
    tip = document.createElement('div'); tip.className = 'tip'; tip.hidden = true; results.appendChild(tip);
    results.querySelectorAll<HTMLButtonElement>('.info-btn').forEach((b) => b.addEventListener('click', () => {
      const p = b.closest('.card')!.querySelector<HTMLElement>('.info-text')!;
      p.hidden = !p.hidden; b.setAttribute('aria-expanded', String(!p.hidden));
    }));
    el.querySelectorAll<HTMLElement>('.run-row').forEach((a) => {
      const on = !all && a.dataset.run === selected.name;
      a.setAttribute('aria-selected', String(on)); a.tabIndex = on ? 0 : -1;
      a.parentElement!.classList.toggle('sel', on);
      if (on) a.scrollIntoView({ block: 'nearest' });
    });
    const body = results.querySelector<HTMLElement>('.card .body');
    if (body && body.clientWidth > 40 && (Math.abs(body.clientWidth - size.w) > 6 || Math.abs(body.clientHeight - size.h) > 6)) {
      size = { w: Math.floor(body.clientWidth), h: Math.floor(body.clientHeight) }; void paint();
    }
  };
  const select = (name: string) => {
    selected = index.runs.find((r) => r.name === name)!; all = false;
    el.querySelector('[data-action="all-runs"]')?.setAttribute('aria-pressed', 'false'); void paint();
  };
  const rowsEls = () => Array.from(el.querySelectorAll<HTMLElement>('.run-row')).filter((r) => r.offsetParent !== null);
  const rowList = el.querySelector<HTMLElement>('.runs');
  rowList?.addEventListener('click', (e) => {
    const row = (e.target as HTMLElement).closest<HTMLElement>('.run-row');
    if (row && !(e.target as HTMLElement).closest('a.open')) { select(row.dataset.run!); row.focus(); }
  });
  rowList?.addEventListener('keydown', (e) => {
    const row = (e.target as HTMLElement).closest<HTMLElement>('.run-row');
    if (!row) return;
    const rs = rowsEls(), i = rs.indexOf(row);
    const to = e.key === 'ArrowDown' ? i + 1 : e.key === 'ArrowUp' ? i - 1 : e.key === 'Home' ? 0 : e.key === 'End' ? rs.length - 1 : -1;
    if (to >= 0) { e.preventDefault(); const t = rs[Math.max(0, Math.min(rs.length - 1, to))]; select(t.dataset.run!); t.focus(); }
    else if (e.key === 'Enter' && row.dataset.open) { e.preventDefault(); location.href = row.dataset.open; }
    else if (e.key === ' ') { e.preventDefault(); select(row.dataset.run!); }
  });
  el.querySelector('[data-action="all-runs"]')?.addEventListener('click', (e) => {
    all = !all; (e.currentTarget as HTMLElement).setAttribute('aria-pressed', String(all)); void paint();
  });
  // hover tooltip: nearest generation on line charts, text on bars
  results.addEventListener('mousemove', (e) => {
    const t = e.target as Element, g = t.closest<SVGGElement>('.barg'), svg = t.closest<SVGSVGElement>('svg.chart[data-tip]');
    const cursor = svg?.querySelector<SVGLineElement>('.cursor');
    let msg = '';
    if (g) msg = g.dataset.tip ?? '';
    else if (svg) {
      const data = JSON.parse(svg.dataset.tip!) as Tip, rect = svg.getBoundingClientRect();
      const i = nearestIndex(e.clientX - rect.left, data.xs.length, Number(svg.dataset.l), Number(svg.dataset.r));
      msg = tipText(data, i);
      const x = Number(svg.dataset.l) + (data.xs.length > 1 ? (i / (data.xs.length - 1)) * (Number(svg.dataset.r) - Number(svg.dataset.l)) : 0);
      cursor?.setAttribute('x1', String(x)); cursor?.setAttribute('x2', String(x)); cursor?.removeAttribute('hidden');
    }
    results.querySelectorAll('.cursor').forEach((c) => { if (c !== cursor) c.setAttribute('hidden', ''); });
    if (!tip) return;
    tip.hidden = !msg; tip.textContent = msg;
    const r = results.getBoundingClientRect();
    tip.style.left = `${Math.min(e.clientX - r.left + 12, r.width - 130)}px`; tip.style.top = `${e.clientY - r.top + 12}px`;
  });
  results.addEventListener('mouseleave', () => { if (tip) tip.hidden = true; results.querySelectorAll('.cursor').forEach((c) => c.setAttribute('hidden', '')); });
  let t: number | undefined;
  new ResizeObserver(() => { clearTimeout(t); t = window.setTimeout(() => { size = DEFAULT_SIZE; void paint(); }, 120); }).observe(results);
  await paint();
}
