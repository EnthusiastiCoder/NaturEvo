// Style-system / icon / tooltip / dashboard requirements for the viewer redesign v2
// (plans/2026-10-20_viewer-redesign-v2.md, "Test cases"). Derived from requirements, not
// from the current layout. Static checks parse viewer/index.html; behaviour checks use the
// shared stub-DOM harness. No dependencies.
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_style_check');
const root = join(dirname(fileURLToPath(import.meta.url)), '..');

// ---------- static helpers ----------
const scripts = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map(m => m[1]);
const styleRaw = (html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1];
const css = styleRaw.replace(/\/\*[\s\S]*?\*\//g, '');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
const attr = (tag, name) => { const m = tag.match(new RegExp('\\b' + name + '\\s*=\\s*(?:"([^"]*)"|\'([^\']*)\')', 'i')); return m ? (m[1] ?? m[2]) : null; };
const stripTags = s => s.replace(/<[^>]*>/g, '').replace(/&nbsp;/g, ' ').trim();

// Flat list of CSS rules (also those nested in @media) -> { sels, body, root }.
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(m => {
  const sels = m[1].split(',').map(s => s.trim().replace(/\s+/g, ' '));
  return { sels, body: m[2], root: sels.length >= 1 && sels.every(s => /^:root(\[data-theme=["']?(light|dark)["']?\])?$/.test(s.trim())) };
});
const decls = body => body.split(';').map(d => { const k = d.indexOf(':'); return k < 0 ? null : [d.slice(0, k).trim().toLowerCase(), d.slice(k + 1).trim()]; }).filter(Boolean);
const rootCss = rules.filter(r => r.root).map(r => r.body).join(';');
const nonRoot = rules.filter(r => !r.root);

// Tooltip elements present in static markup: id -> text.
const tipsStatic = new Map();
for (const m of markup.matchAll(/<(\w+)\b([^>]*\brole\s*=\s*["']tooltip["'][^>]*)>([\s\S]*?)<\/\1>/g)) {
  const id = attr(m[2], 'id'); if (id) tipsStatic.set(id, stripTags(m[3]));
}

// ---------- (1) no emoji / symbol glyphs outside SP and ZN ----------
{
  const stripSprites = s => s.replace(/\b(SP|ZN)\s*=\s*\[\[[\s\S]*?\]\]\s*;/g, '');
  const text = markup + '\n' + scripts.map(stripSprites).join('\n');
  const glyph = /[\p{Extended_Pictographic}\u2190-\u21FF\u25A0-\u25FF\u2600-\u27BF\u2B00-\u2BFF\u2022\u00D7\u2026\uFE0F]/gu;
  const found = [...new Set(text.match(glyph) || [])].map(c => 'U+' + c.codePointAt(0).toString(16).toUpperCase());
  check(found.length === 0, 'emoji/symbol glyph chars outside SP/ZN sprite arrays: ' + found.join(' '));
  const ents = [...new Set([...text.matchAll(/&(?:#(\d+)|#x([0-9a-f]+)|(rarr|larr|uarr|darr|harr|bull|times|hellip|check|cross|starf|star|phone));/gi)]
    .filter(m => m[3] || ((c => (c >= 0x2190 && c <= 0x21ff) || (c >= 0x25a0 && c <= 0x25ff) || (c >= 0x2600 && c <= 0x27bf) || (c >= 0x2b00 && c <= 0x2bff) || c === 0x2022 || c >= 0x1f000)(m[1] ? +m[1] : parseInt(m[2], 16))))
    .map(m => m[0]))];
  check(ents.length === 0, 'symbol HTML entities used as UI chrome: ' + ents.join(' '));
  const esc = [...new Set([...text.matchAll(/\\u(2[1-9a-b][0-9a-f]{2}|2022|d83[cd])/gi)].map(m => m[0]))];
  check(esc.length === 0, 'symbol \\u escapes used as UI chrome: ' + esc.join(' '));
}

// ---------- (2) one inline Lucide sprite, no external assets ----------
const symbols = new Set([...html.matchAll(/<symbol\b[^>]*\bid\s*=\s*["'](i-[\w-]+)["']/g)].map(m => m[1]));
{
  const svgs = [...markup.matchAll(/<svg\b[\s\S]*?<\/svg>/g)].map(m => m[0]).filter(s => /<symbol\b/.test(s));
  check(svgs.length === 1, 'exactly one inline sprite <svg> containing <symbol> required, found ' + svgs.length);
  check(symbols.size >= 10, 'sprite must define >= 10 <symbol id="i-..."> icons, found ' + symbols.size);
  check(/ISC/.test(html) && /lucide/i.test(html), 'Lucide ISC licence notice (mentions "ISC" and "Lucide") must be present in the html');
  const uses = [...html.matchAll(/<use\b[^>]*\bhref\s*=\s*["']#(i-[\w-]+)["']/g)].map(m => m[1]);
  check(uses.length >= 5, 'icons must be used via <use href="#i-name">, found ' + uses.length + ' uses');
  const missing = [...new Set(uses.filter(u => !symbols.has(u)))];
  check(missing.length === 0, '<use> references missing symbols: ' + missing.join(', '));
  check(!/<img\b/i.test(html), 'no <img> allowed');
  check(!/\b(src|href)\s*=\s*["']\s*(https?:)?\/\//i.test(html), 'no external src/href (http(s):// or //) allowed');
  check(!/<link\b/i.test(html), 'no <link> allowed');
}

// ---------- (3) icon-only buttons: aria-label + tooltip ----------
{
  const bad = [];
  let n = 0;
  for (const m of markup.matchAll(/<button\b([^>]*)>([\s\S]*?)<\/button>/g)) {
    n++;
    if (stripTags(m[2]).length > 0) continue;
    const id = attr(m[1], 'id') || '(no id)';
    const label = attr(m[1], 'aria-label');
    const d = attr(m[1], 'aria-describedby');
    if (!label || !label.trim()) bad.push(id + ': no aria-label');
    if (!d || !tipsStatic.has(d.trim())) bad.push(id + ': aria-describedby must point at an existing role=tooltip id, got ' + d);
  }
  check(n > 0, 'no <button> found in markup');
  check(bad.length === 0, 'icon-only buttons need aria-label + tooltip: ' + bad.join('; '));
}

// ---------- (4) info buttons and generated knob tooltips ----------
{
  const infos = [...markup.matchAll(/<button\b([^>]*\bclass\s*=\s*["'][^"']*\binfo\b[^"']*["'][^>]*)>/g)];
  check(infos.length >= 6, 'expected >= 6 static .info buttons (KPIs, controls), found ' + infos.length);
  for (const m of infos) {
    const id = attr(m[1], 'id') || attr(m[1], 'aria-label') || '(info)';
    check(/^About /.test(attr(m[1], 'aria-label') || ''), `.info ${id}: aria-label must start with "About "`);
    const d = attr(m[1], 'aria-describedby');
    check(!!d && tipsStatic.has(d), `.info ${id}: aria-describedby must reference an existing role=tooltip id, got ${d}`);
    check(!!d && (tipsStatic.get(d) || '').length > 0, `.info ${id}: tooltip text must be non-empty`);
    check(/\btype\s*=\s*["']button["']/.test(m[1]), `.info ${id}: must be type="button"`);
  }
}
{
  const KNOBS = [
    { key: 'alpha_k', label: 'Alpha', min: 0.25, max: 7.5, default: 2.5 },
    { key: 'beta_k', label: 'Beta', min: 2, max: 40, default: 12 },
    { key: 'gamma_k', label: 'Gamma', min: 10, max: 900, default: 125 },
  ];
  const fetchImpl = async url => (url.startsWith('/api/config') ? jsonResp({ knobs: KNOBS, max_runs: 20000 }) : jsonResp({ error: 'nf' }, 404));
  const v = load({ protocol: 'http:', run: makeRun(30), fetchImpl });
  await tick();
  const K = v.$('knobs');
  const has = (e, c) => e.classList && e.classList.contains(c);
  // Collect info buttons and tooltips from both the node tree and serialized innerHTML strings.
  const tips = new Map(), infos = [];
  const visit = e => {
    if (has(e, 'info')) infos.push({ d: e['aria-describedby'] || null, l: e['aria-label'] || null });
    if (e.role === 'tooltip' || has(e, 'tip')) tips.set(e._id, (tips.get(e._id) || '') + e.textContent);
    if (typeof e._html === 'string' && e._html) {
      for (const m of e._html.matchAll(/<button\b([^>]*\bclass\s*=\s*["'][^"']*\binfo\b[^"']*["'][^>]*)>/g)) infos.push({ d: attr(m[1], 'aria-describedby'), l: attr(m[1], 'aria-label') });
      for (const m of e._html.matchAll(/<(\w+)\b([^>]*\brole\s*=\s*["']tooltip["'][^>]*)>([\s\S]*?)<\/\1>/g)) { const id = attr(m[2], 'id'); if (id) tips.set(id, stripTags(m[3])); }
    }
    e.children.forEach(visit);
  };
  const box = v.$('run-controls') || K;
  visit(box); visit(K);
  // de-duplicate buttons visited twice (K is inside run-controls only in real DOM)
  const uniq = [...new Map(infos.map(i => [i.d + '|' + i.l, i])).values()];
  check(uniq.length >= KNOBS.length, `each knob needs an info button; found ${uniq.length} for ${KNOBS.length} knobs`);
  KNOBS.forEach((k, i) => {
    const tipText = (uniq.filter(x => x.d && tips.has(x.d)).map(x => tips.get(x.d)).filter(t => t.includes(String(k.min)) && t.includes(String(k.max)) && t.includes(String(k.default))));
    check(tipText.length >= 1, `knob ${k.key}: no tooltip text containing min ${k.min}, max ${k.max} and default ${k.default}`);
  });
  check(uniq.every(x => x.l && /^About /.test(x.l)), 'generated .info buttons need aria-label "About ..."');
}

// ---------- (5) tooltip CSS reveals on hover AND focus ----------
{
  const tipRules = nonRoot.filter(r => r.sels.some(s => /\.tip\b/.test(s)));
  const reveal = (re) => tipRules.some(r => r.sels.some(s => /\.tip\b/.test(s) && re.test(s)));
  check(reveal(/:hover/), 'tooltip CSS must reveal .tip on :hover');
  check(reveal(/:focus(-visible|-within)?/), 'tooltip CSS must reveal .tip on :focus / :focus-visible / :focus-within');
  check(tipRules.some(r => r.sels.every(s => !/:hover|:focus/.test(s)) && decls(r.body).some(([k, v]) => (k === 'visibility' && /hidden/.test(v)) || (k === 'display' && /none/.test(v)) || (k === 'opacity' && /^0/.test(v)))),
    '.tip must be hidden by default (visibility:hidden / display:none / opacity:0)');
  check(/\bEscape\b/.test(scripts.join('\n')), 'inline JS must handle the Escape key to hide tooltips');
}

// ---------- (6) tokens, no ad-hoc colours / px font sizes ----------
{
  const need = ['--bg', '--card', '--border', '--fg', '--muted', '--accent', '--control-h', '--icon', '--r-md', '--s1', '--s2', '--s3', '--s4', '--s5', '--s6', '--fs-sm', '--fs-kpi', '--shadow-pop'];
  const missing = need.filter(t => !new RegExp('(^|[;\\s])' + t + '\\s*:').test(rootCss));
  check(missing.length === 0, ':root is missing tokens: ' + missing.join(' '));
  const colour = /#[0-9a-fA-F]{3,8}\b|\brgba?\s*\(|\bhsla?\s*\(/;
  const badC = [], badF = [];
  for (const r of nonRoot) for (const [k, v] of decls(r.body)) {
    if (colour.test(v)) badC.push(r.sels[0] + ' { ' + k + ': ' + v + ' }');
    if ((k === 'font-size' || k === 'font') && /\d\s*px\b/.test(v)) badF.push(r.sels[0] + ' { ' + k + ': ' + v + ' }');
  }
  check(badC.length === 0, 'hex/rgb/hsl colour literals outside :root: ' + badC.slice(0, 5).join(' | '));
  check(badF.length === 0, 'px font-size outside :root: ' + badF.slice(0, 5).join(' | '));
  const inline = [...markup.matchAll(/\bstyle\s*=\s*["']([^"']*)["']/g)].map(m => m[1]);
  check(inline.length === 0, 'no inline style= attributes in markup, found: ' + inline.slice(0, 3).join(' | '));
  const jsColours = scripts.join('\n').replace(/\b(SP|ZN)\s*=\s*\[\[[\s\S]*?\]\]\s*;/g, '').match(/#[0-9a-fA-F]{3,8}\b(?=[^\w-])|\brgba?\s*\(/g) || [];
  check(jsColours.length === 0, 'colour literals in inline JS (use tokens/classes): ' + jsColours.slice(0, 5).join(' '));
}

// ---------- (7) viewer/STYLE.md documents every component class ----------
{
  const stylePath = join(root, 'viewer', 'STYLE.md');
  check(existsSync(stylePath), 'viewer/STYLE.md must exist');
  if (existsSync(stylePath)) {
    const doc = readFileSync(stylePath, 'utf8');
    const classes = new Set();
    for (const m of markup.matchAll(/\bclass\s*=\s*["']([^"']*)["']/g)) m[1].split(/\s+/).filter(Boolean).forEach(c => classes.add(c));
    const undocumented = [...classes].filter(c => !new RegExp('(^|[^\\w-])\\.?' + c.replace(/[-/\\^$*+?.()|[\]{}]/g, '\\$&') + '($|[^\\w-])').test(doc));
    check(classes.size > 0, 'markup uses no classes (component classes expected)');
    check(undocumented.length === 0, 'classes used in markup but not documented in viewer/STYLE.md: ' + undocumented.join(', '));
    for (const c of ['btn', 'btn-primary', 'btn-outline', 'btn-ghost', 'btn-icon', 'input', 'label', 'card', 'card-title', 'kpi', 'tab', 'tabs', 'badge', 'tip', 'info', 'expander', 'table', 'scroll']) {
      check(new RegExp('(^|[^\\w-])\\.' + c + '($|[^\\w-])').test(doc), `STYLE.md must document .${c}`);
    }
  }
}

// ---------- (8) dashboard renders KPI cards, <= 20 bars, log-scale toggle ----------
{
  const hist = new Array(60).fill(0); hist[3] = 4; hist[10] = 12; hist[11] = 3; hist[59] = 1981; // heavily skewed, 60 levels
  const stats = { runs: 2000, wins: 1000, avg_deaths: 2.5, avg_final_level: 38.5, level_hist: hist };
  const v = load({ protocol: 'file:', run: makeRun(30) });
  v.win.STATS = stats;
  v.$('tab-dashboard').click();
  await tick();
  const D = v.$('dstats'), H = v.$('hist');
  const textOf = e => e.textContent + ' ' + stripTags(e._html || '') + ' ' + e.children.map(c => textOf(c)).join(' ');
  const dText = textOf(D);
  check(/win rate/i.test(dText) && /50(\.0+)?\s*%/.test(dText), 'KPI "Win rate" card must show 50% for 1000/2000 wins, got ' + JSON.stringify(dText.trim().slice(0, 200)));
  check(/avg\.? deaths/i.test(dText) && /2\.5/.test(dText), 'KPI "Avg deaths" must show 2.5');
  check(/avg\.? final level/i.test(dText) && /38\.5/.test(dText), 'KPI "Avg final level" must show 38.5');
  check(/runs/i.test(dText) && /2000|2,000/.test(dText), 'KPI "Runs" must show 2000');
  const kpiTree = e => e.children.reduce((n, c) => n + (c.classList.contains('kpi') ? 1 : 0) + kpiTree(c), 0);
  const cardCount = kpiTree(D) + [...(D._html || '').matchAll(/class\s*=\s*["'][^"']*\bkpi\b/g)].length;
  check(cardCount >= 4, 'at least 4 KPI cards (.kpi) expected in #dstats, found ' + cardCount);

  const isBar = e => e.tagName === 'RECT' || (e._cls || '').split(/\s+/).some(c => c === 'bar' || /-bar$/.test(c));
  const countTree = e => e.children.reduce((n, c) => n + (isBar(c) ? 1 : 0) + countTree(c), 0);
  const countStr = e => ((e._html || '').match(/<rect\b|class\s*=\s*["'][^"']*(?:^|[\s"'])(?:[\w]+-)?bar(?=[\s"'])/g) || []).length;
  const bars = Math.max(countTree(H), countStr(H));
  check(bars >= 2, '#hist must render bars (class "bar" or <rect>), found ' + bars);
  check(bars <= 20, '#hist must bin 60 levels into <= 20 bars, found ' + bars);

  const body = v.$('dashboard');
  const live = [H, D, body].filter(Boolean);
  const toggleHtml = markup.match(/<input\b[^>]*\btype\s*=\s*["']checkbox["'][^>]*>/g) || [];
  const dyn = live.some(e => /type\s*=\s*["']checkbox["']/.test(e._html || '') || e.type === 'checkbox' || e.children.some(function f(c) { return c.type === 'checkbox' || c.children.some(f); }));
  check(toggleHtml.some(t => /log/i.test(t)) || dyn, 'a "Log scale" checkbox (input type=checkbox, id/name containing "log") must exist in the dashboard');
  check(/log scale/i.test(markup + dText + textOf(H)), 'the toggle must be labelled "Log scale"');
}
