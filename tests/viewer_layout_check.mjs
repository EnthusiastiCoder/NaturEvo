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
