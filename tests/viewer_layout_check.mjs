// Layout/structure requirements for the viewer app-shell redesign
// (plans/2026-10-20_viewer-redesign.md). Static checks parse the html text; behaviour
// checks use the shared stub-DOM harness. No dependencies.
import { html, load, makeRun, runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_layout_check');

// ---------- static helpers ----------
const styleText = (html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1].replace(/\/\*[\s\S]*?\*\//g, '');
// Drop @media/@supports blocks so only base rules are considered (brace-depth scan).
function stripAt(css) {
  let out = '', i = 0;
  while (i < css.length) {
    if (css[i] === '@') {
      const o = css.indexOf('{', i);
      if (o < 0) break;
      let d = 1, j = o + 1;
      while (j < css.length && d) { if (css[j] === '{') d++; else if (css[j] === '}') d--; j++; }
      i = j;
    } else { out += css[i++]; }
  }
  return out;
}
const baseCss = stripAt(styleText);
const rules = [...baseCss.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(m => ({
  sels: m[1].split(',').map(s => s.trim().replace(/\s+/g, ' ')), body: m[2],
}));
// Merged declarations of every rule whose selector list contains a selector that
// (a) equals `sel` or (b) ends with `sel` as its last compound (e.g. "details#log-box #log").
function props(sel, { descendants = true } = {}) {
  const p = {};
  for (const r of rules) {
    const hit = r.sels.some(s => s === sel || (descendants && (s.endsWith(' ' + sel) || s.endsWith('>' + sel))));
    if (!hit) continue;
    for (const d of r.body.split(';')) {
      const k = d.indexOf(':'); if (k < 0) continue;
      p[d.slice(0, k).trim().toLowerCase()] = d.slice(k + 1).trim().toLowerCase().replace(/\s*!important/, '');
    }
  }
  return p;
}

const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
const idsInMarkup = new Set([...markup.matchAll(/\bid\s*=\s*["']([^"']+)["']/g)].map(m => m[1]));

// <details> ranges: { id, open, start, end } over `markup`.
function detailsRanges() {
  const out = [];
  for (const m of markup.matchAll(/<details\b([^>]*)>/g)) {
    const end = markup.indexOf('</details>', m.index);
    const idm = m[1].match(/\bid\s*=\s*["']([^"']+)["']/);
    out.push({ id: idm ? idm[1] : null, open: /(^|\s)open(\s|=|$)/.test(m[1]), start: m.index, end: end < 0 ? markup.length : end });
  }
  return out;
}
const dets = detailsRanges();
const posOfId = id => { const m = markup.match(new RegExp('<\\w+[^>]*\\bid\\s*=\\s*["\']' + id + '["\']')); return m ? m.index : -1; };

// ---------- (1) shell: no page scroll ----------
{
  const h = props('html'), b = props('body');
  check(h.overflow === 'hidden' || (h['overflow-x'] === 'hidden' && h['overflow-y'] === 'hidden'), 'CSS html must have overflow:hidden, got ' + JSON.stringify(h.overflow));
  check(b.overflow === 'hidden' || (b['overflow-x'] === 'hidden' && b['overflow-y'] === 'hidden'), 'CSS body must have overflow:hidden, got ' + JSON.stringify(b.overflow));
  check(/^(100dvh|100vh)$/.test(b.height || ''), 'CSS body height must be 100dvh or 100vh (single-viewport shell), got ' + JSON.stringify(b.height));
}

// ---------- (2) long-content panels scroll inside a bounded box ----------
{
  // #knobs is intentionally NOT a scroller (Addendum 2: knob panel must not scroll); see viewer_clip_check.mjs
  const spec = [['log', ['log-box']], ['hist', ['dashboard']]];
  for (const [id, containers] of spec) {
    const p = props('#' + id);
    const cp = containers.map(c => props('#' + c));
    const ov = x => /^(auto|scroll)$/.test(x.overflow || x['overflow-y'] || '');
    check(ov(p) || cp.some(ov), `#${id} (or its drawer container) must have overflow:auto|scroll, got ${JSON.stringify(p.overflow)}`);
    const bounded = x => x['max-height'] || x.height || x['min-height'] === '0' || x['min-height'] === '0px' || x.flex || x['grid-row'];
    check(bounded(p) || cp.some(x => x['max-height']), `#${id} must be height-bounded (max-height/height/min-height:0/flex), got ${JSON.stringify(p)}`);
  }
}

// ---------- (3) drawers are closed <details> ----------
for (const id of ['knob-box', 'log-box', 'more-box', 'batch-box']) {
  const d = dets.find(x => x.id === id);
  check(!!d, `#${id} must exist as a <details>`);
  if (d) check(!d.open, `#${id} must not have the open attribute (closed by default)`);
}

// ---------- (4) containment ----------
{
  const within = (outer, inner) => {
    const d = dets.find(x => x.id === outer); const p = posOfId(inner);
    return !!d && p > d.start && p < d.end;
  };
  check(within('log-box', 'log'), '#log must be inside #log-box');
  check(within('knob-box', 'knobs'), '#knobs must be inside #knob-box');
}

// ---------- (5) cognitive-load cap on visible inputs; seed width ----------
{
  const bad = [];
  for (const m of markup.matchAll(/<input\b([^>]*)>/g)) {
    const t = (m[1].match(/\btype\s*=\s*["']?(\w+)/) || [, 'text'])[1].toLowerCase();
    if (t !== 'text' && t !== 'number') continue;
    const id = (m[1].match(/\bid\s*=\s*["']([^"']+)["']/) || [, '(no id)'])[1];
    if (id === 'seed') continue;
    const inClosed = dets.some(d => !d.open && m.index > d.start && m.index < d.end);
    if (!inClosed) bad.push(id);
  }
  check(bad.length === 0, 'text/number inputs outside a closed <details> (only #seed allowed): ' + bad.join(', '));
  // v2: the dashboard "Log scale" checkbox is the only other default-visible input (range #scrub is not text/number).
  const badCb = [];
  for (const m of markup.matchAll(/<input\b([^>]*)>/g)) {
    if (!/\btype\s*=\s*["']?checkbox/i.test(m[1])) continue;
    const id = (m[1].match(/\bid\s*=\s*["']([^"']+)["']/) || [, '(no id)'])[1];
    const inClosed = dets.some(d => !d.open && m.index > d.start && m.index < d.end);
    if (!inClosed && !/log/i.test(id)) badCb.push(id);
  }
  check(badCb.length === 0, 'checkbox inputs outside a closed <details> must be the log-scale toggle (id contains "log"): ' + badCb.join(', '));
  const w = props('#seed').width || '';
  const ch = /^([\d.]+)ch$/.exec(w), px = /^([\d.]+)px$/.exec(w);
  check((ch && +ch[1] >= 12) || (px && +px[1] >= 100), '#seed CSS width must be >= 12ch (10-digit seed), got ' + JSON.stringify(w));
}

// ---------- (6) no fixed-width timeline/body, no narrow cap ----------
{
  const tw = props('#timeline').width || '';
  check(!/px$/.test(tw), '#timeline CSS width must not be a fixed pixel value, got ' + tw);
  const b = props('body');
  const mw = b['max-width'];
  check(!mw || mw === 'none' || (/px$/.test(mw) && parseFloat(mw) >= 1600) || /%$|vw$/.test(mw), 'body max-width must not cap below 1600px, got ' + mw);
  check(!/px$/.test(b.width || ''), 'body must not have fixed pixel width, got ' + b.width);
}

// ---------- (7) summary strip ids ----------
for (const id of ['summary', 'outcome', 'sum-level', 'sum-hp', 'sum-zone', 'sum-progress']) {
  check(idsInMarkup.has(id), `summary id #${id} missing from markup`);
}

// ---------- (8) summary behaviour via harness ----------
{
  const mk = (n, won, lastLevel) => {
    const r = makeRun(n, 4242); r.won = won;
    r.events[n - 1].hero.level = lastLevel; r.events[n - 1].hero.hp = 7;
    return r;
  };
  const txt = v => id => (v.$(id) ? v.$(id).textContent : '');
  let v;
  try { v = load({ protocol: 'file:', run: makeRun(300) }); } catch (e) { check(false, 'script load threw: ' + e.message); done(); }
  const T = txt(v);
  const lost = mk(100, false, 9), won = mk(100, true, 12);
  v.win.__jv.setRun(lost);
  v.win.__jv.goto(40);
  check(/3/.test(T('sum-level')), 'sum-level at event 40 (level 1+floor(40/20)=3) must contain "3", got ' + JSON.stringify(T('sum-level')));
  check(/40/.test(T('sum-progress')), 'sum-progress at event 40 must contain 40, got ' + JSON.stringify(T('sum-progress')));
  check(/99/.test(T('sum-progress')) || /100/.test(T('sum-progress')), 'sum-progress must show the total (99 or 100), got ' + JSON.stringify(T('sum-progress')));
  check(/in progress/i.test(T('outcome')) && /40/.test(T('outcome')), 'outcome mid-run must read "in progress at event 40", got ' + JSON.stringify(T('outcome')));
  check(!/\b(WON|LOST)\b/.test(T('outcome')), 'outcome mid-run must not claim WON/LOST, got ' + JSON.stringify(T('outcome')));
