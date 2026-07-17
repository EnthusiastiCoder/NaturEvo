import { load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_postgame_check');
const hidden = el => !!el && el.classList.contains('hidden');
const shown = el => !!el && !hidden(el) && el.style.display !== 'none';

const ev = (i, extra = {}) => ({
  idx: i, kind: 'encounter', text: 'event ' + i,
  hero: { level: 1 + Math.floor(i / 20), hp: 10, max_hp: 10, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0 },
  mon: null, ...extra,
});
const mon = stage => ({ species: 0, level: 5, hp: 8, max_hp: 10, ...(stage ? { stage } : {}) });
