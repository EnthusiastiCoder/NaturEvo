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

// ---- file:// : hidden, no fetch
{
  const v = load({ protocol: 'file:', run: makeRun(30) });
  await tick();
  check(v.$('run-controls'), '#run-controls element missing');
  check(hidden(v.$('run-controls')), 'file://: #run-controls must stay hidden');
  check(v.fetchCalls.length === 0, 'file://: no fetch must be made, got ' + JSON.stringify(v.fetchCalls));
}

// ---- http with stubbed fetch
let pendingRun = null;
const newRun = makeRun(80, 7);
let mode = 'ok';
const fetchImpl = async (url) => {
  if (url.startsWith('/api/config')) return jsonResp(CONFIG);
  if (mode === 'error') return jsonResp({ error: 'runs must be 1..20000' }, 400);
  if (url.startsWith('/api/run')) {
    if (mode === 'hold') return new Promise(res => { pendingRun = () => res(jsonResp(newRun)); });
    return jsonResp(newRun);
  }
  if (url.startsWith('/api/batch')) return jsonResp({ runs: 123, wins: 60, avg_deaths: 1.5, avg_final_level: 30.2, level_hist: [0, 3, 5, 9] });
  return jsonResp({ error: 'nf' }, 404);
};
const v = load({ protocol: 'http:', run: makeRun(300), fetchImpl });
await tick();
const $ = v.$;
