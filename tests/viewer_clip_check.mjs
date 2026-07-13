// Addendum 2 requirements (plans/2026-10-20_viewer-redesign-v2.md): no clipped content,
// consistent right-aligned selects, tuning knobs as range + synced number input in a
// non-scrolling multi-column grid. Derived from requirements; static CSS/markup checks plus
// behaviour checks with the shared stub-DOM harness. Real pixel checks: tests/viewer_layout_checklist.md section 15.
// UPDATED for Addendum 3b: #speed is no longer a <select> (log range slider + number input). The "Speed select
// exists" and "#speed must use class select" checks were replaced; any remaining selects must still use .select.
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_clip_check');

const css = ((html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1]).replace(/\/\*[\s\S]*?\*\//g, '');
const scripts = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map(m => m[1]).join('\n');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(m => ({
  sels: m[1].split(',').map(s => s.trim().replace(/\s+/g, ' ')),
  decl: Object.fromEntries(m[2].split(';').map(d => { const k = d.indexOf(':'); return k < 0 ? null : [d.slice(0, k).trim().toLowerCase(), d.slice(k + 1).trim()]; }).filter(Boolean)),
}));
const rulesFor = pred => rules.filter(r => r.sels.some(pred));
const prop = (pred, name) => rulesFor(pred).map(r => r.decl[name]).filter(v => v !== undefined);

// ---------- (1) no-clipping structure ----------
{
  const body = prop(s => s === 'body', 'height');
  check(body.some(v => /100dvh/.test(v)), 'body height must be 100dvh, got ' + JSON.stringify(body));
  const rows = prop(s => s === '.app' || s === '.shell' || s === '#app', 'grid-template-rows');
  check(rows.some(v => /minmax\(\s*0\s*,\s*1fr\s*\)/.test(v)), 'shell grid rows must use minmax(0,1fr) to reallocate height, got ' + JSON.stringify(rows));
  check(prop(s => s === '.main', 'min-height').some(v => /^0(px)?$/.test(v)), '.main (growing area) needs min-height:0');
  check(prop(s => s === '.view', 'min-height').some(v => /^0(px)?$/.test(v)), '.view needs min-height:0');
  check(prop(s => s === '#timeline-card', 'min-height').some(v => /^0(px)?$/.test(v)), '#timeline-card (growing card) needs min-height:0');
  // overflow:hidden/clip only on shell elements, .sr, .bar or ellipsis rules
  const allowed = new Set(['html', 'body', 'html,body', '.app', '.shell', '#app', '.main', '.sr', '.bar', '.view']);
  const bad = [];
  for (const r of rules) {
    const o = (r.decl.overflow || '') + ' ' + (r.decl['overflow-x'] || '') + ' ' + (r.decl['overflow-y'] || '');
    if (!/\b(hidden|clip)\b/.test(o)) continue;
    if (/ellipsis/.test(r.decl['text-overflow'] || '')) continue;
    for (const s of r.sels) if (!allowed.has(s)) bad.push(s);
  }
  check(bad.length === 0, 'overflow:hidden/clip on non-shell, non-ellipsis elements (clips content): ' + [...new Set(bad)].join(', '));
  // vh literals: none above 100, and children must not size themselves in vh beyond max-height caps
  const vhBad = [];
  for (const r of rules) for (const [k, v] of Object.entries(r.decl)) {
    for (const m of v.matchAll(/(-?[\d.]+)\s*vh\b/g)) {
      if (+m[1] > 100) vhBad.push(r.sels[0] + ' ' + k + ':' + v);
      else if (/^(height|min-height)$/.test(k) && !r.sels.every(s => s === 'html' || s === 'body' || s === 'html,body')) vhBad.push(r.sels[0] + ' ' + k + ':' + v);
    }
  }
  check(vhBad.length === 0, 'vh sizing that can exceed the shell (use %/dvh/grid rows): ' + vhBad.join(' | '));
}
