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

// ---------- (2) selects: one .select class, right aligned, same chevron padding ----------
{
  const tags = [...markup.matchAll(/<select\b([^>]*)>/g)].map(m => m[1]).concat([...scripts.matchAll(/<select\b([^>]*)>/g)].map(m => m[1]));
  // (3b) no minimum count: speed is no longer a select; any selects that remain must use .select
  const noClass = tags.filter(t => !/\bclass\s*=\s*["'][^"']*\bselect\b/.test(t));
  check(noClass.length === 0, 'every <select> needs class "select": ' + noClass.join(' | '));
  check(!/createElement\(\s*["']select["']\s*\)/.test(scripts) || /\.className\s*=\s*["'][^"']*\bselect\b|classList\.add\(\s*["']select["']/.test(scripts), 'script-created selects must get class select');
  const sel = rulesFor(s => s === '.select' || s === 'select.select');
  check(sel.length >= 1, 'a .select CSS rule is required');
  const d = Object.assign({}, ...sel.map(r => r.decl));
  check(/^right$/.test(d['text-align'] || ''), '.select text-align must be right, got ' + d['text-align']);
  check(/^right$/.test(d['text-align-last'] || ''), '.select text-align-last must be right, got ' + d['text-align-last']);
  check(/^none$/.test(d.appearance || d['-webkit-appearance'] || ''), '.select needs appearance:none for the custom chevron');
  check(!!(d['padding-right'] || d['padding-inline-end'] || d.padding), '.select must define right/inline-end padding for the chevron');
  const bgChevron = /url\(/.test(d['background-image'] || d.background || '');
  const useCount = (markup.match(/<use\b[^>]*href\s*=\s*["']#i-chevron-down["']/g) || []).length;
  check(bgChevron || useCount >= tags.length, 'chevron-down must come from .select background-image url(...) or a sibling <use href="#i-chevron-down"> per select');
  const other = rules.filter(r => r.sels.some(s => /(^|[\s#.])(speed)\b|^select\b/.test(s) && s !== '.select' && s !== 'select.select') && /padding/.test(Object.keys(r.decl).join(' ')));
  check(other.length === 0, 'no per-select padding overrides (all selects share .select padding): ' + other.map(r => r.sels[0]).join(', '));
  check(!/<select\b[^>]*\bid\s*=\s*["']speed["']/.test(markup), '#speed must not be a select any more (addendum 3b)');
}

// ---------- (3) knobs: range + synced number ----------
const CONFIG = {
  knobs: [
    { key: 'boss_attempts', label: 'Boss attempts', min: 1, max: 50, default: 8 },
    { key: 'potion_cost', label: 'Potion cost', min: 1, max: 500, default: 25 },
    { key: 'monster_hp_mult', label: 'Monster HP x', min: 0.1, max: 5.0, default: 1.0 },
    { key: 'crit_chance', label: 'Crit chance', min: 0, max: 1, default: 0.25 },
    { key: 'xp_rate', label: 'XP rate', min: 0, max: 10, default: 2.5 },
  ],
  max_runs: 20000,
};
const newRun = makeRun(40, 3);
const fetchImpl = async url => {
  if (url.startsWith('/api/config')) return jsonResp(CONFIG);
  if (url.startsWith('/api/run')) return jsonResp(newRun);
  return jsonResp({ error: 'nf' }, 404);
};
const v = load({ protocol: 'http:', run: makeRun(30), fetchImpl });
await tick();
const $ = v.$;
const rng = k => $('knob-' + k), num = k => $('knob-' + k + '-num');
const setFire = (el, val, ...types) => { el.value = String(val); types.forEach(t => el.fire(t)); };
const edit = (el, val) => setFire(el, val, 'input', 'change');
const n = el => Number(el.value);

for (const k of CONFIG.knobs) {
  check(rng(k.key) && rng(k.key).type === 'range', `#knob-${k.key} must be <input type=range>, got ${rng(k.key) && rng(k.key).type}`);
  check(num(k.key) && num(k.key).type === 'number', `#knob-${k.key}-num must be <input type=number>, got ${num(k.key) && num(k.key).type}`);
  if (!rng(k.key) || !num(k.key)) continue;
  check(String(rng(k.key).min) === String(k.min) && String(rng(k.key).max) === String(k.max), `range min/max for ${k.key}: ${rng(k.key).min}..${rng(k.key).max}`);
  check(String(num(k.key).min) === String(k.min) && String(num(k.key).max) === String(k.max), `number min/max for ${k.key}`);
  check(n(rng(k.key)) === k.default && n(num(k.key)) === k.default, `${k.key} both inputs show default ${k.default}, got ${rng(k.key).value}/${num(k.key).value}`);
}
const stepOf = k => rng(k) && Number(rng(k).step);
check(stepOf('boss_attempts') === 1 && stepOf('potion_cost') === 1, 'integer knobs step 1, got ' + stepOf('boss_attempts') + '/' + stepOf('potion_cost'));
check(stepOf('monster_hp_mult') === 0.1, 'fractional range 4.9 (<=10) step 0.1, got ' + stepOf('monster_hp_mult'));
check(stepOf('xp_rate') === 0.1, 'integer min/max but fractional default, range 10: step 0.1, got ' + stepOf('xp_rate'));
check(stepOf('crit_chance') === 0.01, 'range <=1 fractional: step 0.01, got ' + stepOf('crit_chance'));
check(num('boss_attempts') && Number(num('boss_attempts').step) === stepOf('boss_attempts'), 'number input step equals range step');

if (rng('boss_attempts') && num('boss_attempts') && $('run-btn')) {
  // slider -> number
  setFire(rng('boss_attempts'), 12, 'input');
  check(n(num('boss_attempts')) === 12, 'slider input must update number input, got ' + num('boss_attempts').value);
  // number -> slider
  setFire(num('boss_attempts'), 20, 'input');
  check(n(rng('boss_attempts')) === 20, 'number input must update slider, got ' + rng('boss_attempts').value);
  // clamping
  edit(num('boss_attempts'), 999);
  check(n(num('boss_attempts')) === 50 && n(rng('boss_attempts')) === 50, `clamp above max to 50, got num ${num('boss_attempts').value} range ${rng('boss_attempts').value}`);
  edit(num('boss_attempts'), -5);
  check(n(num('boss_attempts')) === 1 && n(rng('boss_attempts')) === 1, `clamp below min to 1, got num ${num('boss_attempts').value} range ${rng('boss_attempts').value}`);
  // step rule: integer knob rounds to integer
  edit(num('boss_attempts'), 7.6);
  check(Number.isInteger(n(num('boss_attempts'))) && Number.isInteger(n(rng('boss_attempts'))) && n(num('boss_attempts')) === n(rng('boss_attempts')), `integer knob must snap to an integer, got ${num('boss_attempts').value}/${rng('boss_attempts').value}`);
  // fractional knob stays within step/2 of typed value and in range
  edit(num('monster_hp_mult'), 1.234);
  check(Math.abs(n(num('monster_hp_mult')) - 1.234) <= 0.05 + 1e-9 && n(num('monster_hp_mult')) === n(rng('monster_hp_mult')), `fractional knob snaps to step 0.1, got ${num('monster_hp_mult').value}/${rng('monster_hp_mult').value}`);
  edit(num('crit_chance'), 7);
  check(n(num('crit_chance')) === 1 && n(rng('crit_chance')) === 1, 'fractional knob clamps to max 1, got ' + num('crit_chance').value);

  // Run query uses the values (number edit and slider edit)
  edit(num('boss_attempts'), 3);
  setFire(rng('potion_cost'), 77, 'input');
  setFire(rng('monster_hp_mult'), 2.5, 'input');
  $('seed').value = '5';
  $('run-btn').click();
  await tick();
  const u = v.fetchCalls.find(x => x.startsWith('/api/run')) || '';
  check(/[?&]boss_attempts=3(&|$)/.test(u), 'Run uses number-edited value boss_attempts=3, got ' + u);
  check(/[?&]potion_cost=77(&|$)/.test(u), 'Run uses slider-edited value potion_cost=77, got ' + u);
  check(/[?&]monster_hp_mult=2\.5(&|$)/.test(u), 'Run uses monster_hp_mult=2.5, got ' + u);
  check(!/knob-|-num/.test(u), 'query keys are the plain knob keys, got ' + u);

  // Reset restores both inputs
  $('knobs-reset').click();
  for (const k of CONFIG.knobs) check(n(rng(k.key)) === k.default && n(num(k.key)) === k.default, `reset must restore both inputs of ${k.key} to ${k.default}, got ${rng(k.key).value}/${num(k.key).value}`);
}
