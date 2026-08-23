// Permadeath toggle in the Run controls (plans/2026-10-21_hero-map-walking.md, Addendum 4 part 1, viewer).
// Written first, failing. Stub-DOM harness conventions (_viewer_harness.mjs).
//
// Names picked where the plan is silent:
//   static markup inside #run-controls: element id="permadeath-toggle" with role="switch" and a tabindex or button tag.
//   aria-checked is "true" or "false", default "true". Click toggles; if the element is not a <button>, keydown
//   with key " " and "Enter" must toggle it too (a <button> gets this natively, tag check stands in for it).
//   Run and batch requests carry permadeath=true or permadeath=false. State persists in localStorage key
//   "jv-permadeath" ("true"/"false") and is restored on load.
//   /api/config lists permadeath with kind "bool"; the viewer must not render a range slider for it (#knob-permadeath absent).
//   Styling uses tokens: CSS rules for #permadeath-toggle / [role="switch"] use var(--...) and no hex literals,
//   so both themes work through the existing theme tokens.
import { html, load, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_permadeath_toggle_check');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '');
const tag = (markup.match(/<(\w+)[^>]*\bid\s*=\s*["']permadeath-toggle["'][^>]*>/) || [])[0] || '';
check(tag, '#permadeath-toggle missing from the markup');
check(/\brole\s*=\s*["']switch["']/.test(tag), '#permadeath-toggle must have role="switch", got ' + tag.slice(0, 120));
const isButton = /^<button\b/i.test(tag);
check(isButton || /\btabindex\s*=/.test(tag), '#permadeath-toggle must be focusable (button or tabindex)');
const inRunControls = (() => { const i = markup.indexOf('id="run-controls"'), j = markup.indexOf('id="permadeath-toggle"'); const end = markup.indexOf('id="run-status"', i); return i >= 0 && j > i && (end < 0 || j < end + 40); })();
check(inRunControls, '#permadeath-toggle must sit inside #run-controls');

const css = ((html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1]).replace(/\/\*[\s\S]*?\*\//g, '');
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].filter(m => /permadeath-toggle|\[role=["']?switch["']?\]|\.switch\b/.test(m[1]));
check(rules.length > 0, 'CSS rules for the switch missing');
check(rules.length > 0 && rules.some(m => /var\(--/.test(m[2])), 'switch CSS must use theme tokens (var(--...))');
check(!rules.some(m => /#[0-9a-fA-F]{3,8}\b/.test(m[2])), 'switch CSS must not use hex literals, tokens only: ' + rules.map(m => m[2]).join(' | ').slice(0, 160));

const config = { knobs: [
  { key: 'monster_hp_mult', label: 'Monster HP multiplier', kind: 'number', min: 0.1, max: 5, default: 1 },
  { key: 'permadeath', label: 'Permadeath', kind: 'bool', default: true },
], max_runs: 20000, max_events: 1048576 };
const RUN = { seed: 1, won: false, deaths: 0, final_level: 1, w: 0, h: 0, steps: 0, total: 1, events: [{ idx: 0, kind: 'travel', text: 'e', hero: { level: 1, hp: 10, max_hp: 10, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0, x: 1, y: 1 }, mon: null, npcs: [] }] };
const fetchImpl = async url => url.startsWith('/api/config') ? jsonResp(config) : url.startsWith('/api/run') ? jsonResp(RUN) : url.startsWith('/api/batch') ? jsonResp({ runs: 1, wins: 0, avg_deaths: 0, avg_final_level: 1, level_hist: [0, 1] }) : jsonResp({ error: 'nf' }, 404);
const boot = async (storage = {}) => { const v = load({ protocol: 'http:', run: RUN, fetchImpl, storage }); await tick(); return v; };
const checked = v => v.$('permadeath-toggle') && v.$('permadeath-toggle').getAttribute('aria-checked');
const key = (el, k) => el.fire('keydown', { key: k, code: k === ' ' ? 'Space' : k });
