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
