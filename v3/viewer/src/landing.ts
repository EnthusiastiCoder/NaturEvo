// Landing view: run picker plus aggregate results. View-model and HTML are pure; mountLanding does the DOM and fetching.
import { type RunIndex, type RunInfo, type Row, parseTrainingCsv } from './runs-index';
import { type EvalRow, aggregateEval, aggregateSeries, defenceShares } from './aggregate';
import { type Size, barChart, legendHtml, lineChart, nearestIndex, pct, tipText, type Tip } from './charts';
import { formatDate, humanName } from './run-labels';
import { DEFAULT_ACCENT, applyTheme, nextTheme, setAccent, themeIconPath, type Theme } from './theme';

export const EMPTY_COMMANDS: string[] = [
  'cargo run --release -- train --out ../runs/slice_s1 --gens 200 --seed 1',
  'cargo run --release -- train --out ../runs/slice_s2 --gens 200 --seed 2',
  'cargo run --release -- train --out ../runs/slice_s3 --gens 200 --seed 3',
];

export type LandingRow = { name: string; label: string; seed: string; dateLabel: string; generations: number; openHref: string; canOpen: boolean };
export type LandingVM = { empty: boolean; rows: LandingRow[] };

/** Opens the latest fight generation of a run in the fight view. */
export function openHrefOf(r: Pick<RunInfo, 'name' | 'fightGens'>): string {
  const g = r.fightGens ?? [];
  return g.length ? `#/fight/${encodeURIComponent(r.name)}/${Math.max(...g)}` : '';
}

export const landingViewModel = (index: RunIndex): LandingVM => ({
  empty: index.runs.length === 0,
  rows: index.runs.map((r) => {
    const href = openHrefOf(r);
    return { name: r.name, label: humanName(r.name), seed: r.seed, dateLabel: formatDate(r.date), generations: r.generations, openHref: href, canOpen: href !== '' };
  }),
});

const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const icon = (d: string) => `<svg viewBox="0 0 24 24" aria-hidden="true">${d}</svg>`;
const ICON = {
  play: '<path d="m6 4 14 8-14 8z"/>',
  layers: '<path d="m12 3 9 5-9 5-9-5z"/><path d="m3 13 9 5 9-5"/>',
  info: '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>',
};
const infoButton = (text: string) => `<button type="button" class="info-btn" aria-label="About this chart" aria-expanded="false" title="${esc(text)}">${icon(ICON.info)}</button>`;

export const ALL = 'all';

function rowHtml(r: LandingRow, selected: boolean): string {
  const meta = [`seed ${r.seed}`, r.generations ? `${r.generations} gen` : '', r.dateLabel].filter(Boolean).join(' · ');
  return `<li class="${selected ? 'sel' : ''}"><div class="run-row" role="option" tabindex="${selected ? 0 : -1}" aria-selected="${selected}" data-run="${esc(r.name)}" title="${esc(r.name)}">
<b class="nm">${esc(r.label)}</b><span class="meta">${esc(meta)}</span></div></li>`;
}

export function renderLandingHtml(vm: LandingVM, selected: string = ALL): string {
  if (vm.empty) {
    return `<section class="landing-empty"><h2>No runs yet</h2>
<p>Train a population from the <code>v3/sim</code> folder; it writes into <code>v3/runs/</code>, which is where this viewer looks.</p>
<ol class="cmds">${EMPTY_COMMANDS.map((c) => `<li><code>${esc(c)}</code></li>`).join('')}</ol></section>`;
  }
  const sel = selected === ALL || !vm.rows.some((r) => r.name === selected) ? ALL : selected;
  const watch = (vm.rows.find((r) => r.name === sel) ?? vm.rows.find((r) => r.canOpen))?.openHref ?? '';
  return `<section class="picker"><div class="head"><h2>Runs</h2>
<button type="button" class="primary" data-action="watch" data-open="${esc(watch)}"${watch ? '' : ' disabled'} title="Watch the latest fight of the selected run">${icon(ICON.play)}Watch fight</button></div>
<div class="runs" role="listbox" aria-label="Runs"><ul>
<li class="${sel === ALL ? 'sel' : ''}"><div class="run-row" role="option" tabindex="${sel === ALL ? 0 : -1}" aria-selected="${sel === ALL}" data-run="${ALL}" title="Average and range over every seed">${icon(ICON.layers)}<b class="nm">All seeds</b><span class="meta">${vm.rows.length} runs</span></div></li>
${vm.rows.map((r) => rowHtml(r, r.name === sel)).join('')}</ul></div></section>
<section class="results" id="landing-results" aria-live="polite"></section>`;
}

// ---- results ----
export type RunData = { run: RunInfo; rows: Row[]; shares: { parry: number[]; block: number[]; dodge: number[] } };
const DEFAULT_SIZE: Size = { w: 240, h: 130 };

const card = (title: string, info: string, body: string, legend = '') =>
  `<article class="card"><header><h3>${esc(title)}</h3>${infoButton(info)}${legend}</header><p class="info-text" hidden>${esc(info)}</p><div class="body">${body}</div></article>`;
const tile = (label: string, info: string, big: string, sub: string) =>
  `<div class="tile" title="${esc(info)}"><span class="tl">${esc(label)}</span><span class="big">${big}</span><span class="sub">${esc(sub)}</span></div>`;

export function renderResultsHtml(data: RunData[], evals: (EvalRow & { run: string })[], title: string, size: Size = DEFAULT_SIZE): string {
  const col = (c: string) => data.map((d) => d.rows.map((r) => r[c] ?? 0));
  const hero = aggregateSeries(col('hero_best')), mon = aggregateSeries(col('monster_best'));
  const yMax = Math.max(1, ...hero.max, ...mon.max);
  const cards: string[] = [];
  const multi = data.length > 1;
  cards.push(card('Learning curves', multi ? 'Best fitness per generation: line is the mean over seeds, shaded area the min to max.' : 'Best fitness per generation for each side. Rising lines mean both sides keep learning.',
    lineChart([{ key: 'hero', label: 'hero', ys: hero.mean, lo: hero.min, hi: hero.max }, { key: 'monster', label: 'monster', ys: mon.mean, lo: mon.min, hi: mon.max }], hero.gens, yMax, size),
    legendHtml([{ key: 'hero', label: 'hero' }, { key: 'monster', label: 'monster' }])));
  const n = data.length ? Math.min(...data.map((d) => d.shares.parry.length)) : 0;
  const sh = (k: 'parry' | 'block' | 'dodge') => aggregateSeries(data.map((d) => d.shares[k]));
  const xs = data[0]?.run.fightGens.slice(0, n) ?? [];
  const p = sh('parry'), b = sh('block'), dg = sh('dodge');
  cards.push(card('Defence mix', 'Share of successful defences that were parries, shield blocks or dodges, in the sampled fight of each generation (mean over seeds).',
    lineChart([{ key: 'hero', label: 'parry', ys: p.mean }, { key: 'accent', label: 'block', ys: b.mean }, { key: 'monster', label: 'dodge', ys: dg.mean }], xs, 1, size, { pct: true }),
    legendHtml([{ key: 'hero', label: 'parry' }, { key: 'accent', label: 'block' }, { key: 'monster', label: 'dodge' }])));
  const bars = evals.map((e) => ({ label: e.run.replace(/^slice_/, ''), value: e.win_rate_hof, tip: `${e.run}\nwin rate vs earlier champions ${pct(e.win_rate_hof)}` }));
  cards.push(card('Win rate', 'Final champions against earlier hall-of-fame champions. Above the dashed line means the late champion wins more often.',
    bars.length ? barChart(bars, size) : '<div class="na">no eval yet</div>'));
  const a = aggregateEval(evals);
  const rng = (r: { mean: number; min: number; max: number }) => (evals.length > 1 ? `range ${pct(r.min)} to ${pct(r.max)}` : 'single run');
  const tiles = evals.length ? [
    tile('Win rate', 'Final champions vs earlier champions', pct(a.win_rate_hof.mean), rng(a.win_rate_hof)),
    tile('Defence rate', 'Share of incoming attacks that were parried, blocked or dodged', pct(a.defence_rate.mean), rng(a.defence_rate)),
    tile('Draws', 'Share of duels with no winner', pct(a.draw_share.mean), rng(a.draw_share)),
    tile('Idle guard', 'Share of time spent guarding with no threat. Low is good.', pct(a.idle_guard.mean), rng(a.idle_guard)),
  ].join('') : '';
  return `<div class="results-head"><h2>${esc(title)}</h2><span class="sub">${multi ? `mean of ${data.length} seeds, band is min to max` : ''}</span></div>${tiles ? `<div class="stats">${tiles}</div>` : ''}<div class="grid">${cards.join('')}</div>`;
}

// ---- DOM controller ----
export async function mountLanding(el: HTMLElement, index: RunIndex): Promise<void> {
  const vm = landingViewModel(index);
  let sel = ALL;
  const root = document.documentElement;
  let theme: Theme = (root.dataset.theme as Theme) || 'dark';
  const bar = document.createElement('header');
  bar.className = 'bar';
  bar.innerHTML = `<h1>NaturEvo v3</h1><span class="spacer"></span>
<input id="ld-accent" type="color" value="${DEFAULT_ACCENT}" title="Accent colour" aria-label="Accent colour">
<button id="ld-theme" type="button" title="Toggle light or dark theme" aria-label="Toggle theme"></button>`;
  const body = document.createElement('div');
  body.className = 'ld-body';
  el.replaceChildren(bar, body);
  const syncTheme = () => { applyTheme(root, theme); bar.querySelector('#ld-theme')!.innerHTML = icon(themeIconPath(theme)); };
  syncTheme();
  bar.querySelector<HTMLInputElement>('#ld-accent')!.oninput = (e) => setAccent(root, (e.target as HTMLInputElement).value);
  bar.querySelector<HTMLElement>('#ld-theme')!.onclick = () => { theme = nextTheme(theme); syncTheme(); };
  body.innerHTML = renderLandingHtml(vm, sel);
  const results = body.querySelector<HTMLElement>('#landing-results');
  if (!results) return;

  const text = async (u: string) => { try { const r = await fetch(u); return r.ok ? await r.text() : ''; } catch { return ''; } };
  const json = async (u: string) => { try { return JSON.parse(await text(u)); } catch { return null; } };
  const cache = new Map<string, RunData>();
  const dataOf = async (run: RunInfo): Promise<RunData> => {
    if (!cache.has(run.name)) {
      const rows = parseTrainingCsv(await text(`/runs/${encodeURIComponent(run.name)}/training.csv`));
      const fights = await Promise.all(run.fightGens.map((g) => json(`/runs/${encodeURIComponent(run.name)}/fight_gen_${g}.json`)));
      const shares = { parry: [] as number[], block: [] as number[], dodge: [] as number[] };
      for (const f of fights) { const s = defenceShares(f?.result?.defence ?? []); shares.parry.push(s.parry); shares.block.push(s.block); shares.dodge.push(s.dodge); }
      cache.set(run.name, { run, rows, shares });
    }
    return cache.get(run.name)!;
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
