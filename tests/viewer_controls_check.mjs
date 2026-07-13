import { load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_controls_check');

const CONFIG = {
  knobs: [
    { key: 'monster_hp_mult', label: 'Monster HP x', min: 0.1, max: 5.0, default: 1.0 },
    { key: 'boss_attempts', label: 'Boss attempts', min: 1, max: 50, default: 8 },
    { key: 'potion_cost', label: 'Potion cost', min: 1, max: 500, default: 25 },
  ],
  max_runs: 20000,
};
const hidden = el => !!el && el.classList.contains('hidden');
