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

check(v.fetchCalls[0] && v.fetchCalls[0].startsWith('/api/config'), 'must fetch /api/config on http load, got ' + JSON.stringify(v.fetchCalls));
check(!hidden($('run-controls')), 'http: #run-controls must be shown after config loads');
for (const k of CONFIG.knobs) {
  const el = $('knob-' + k.key);
  check(el, 'missing input #knob-' + k.key);
  check(el && String(el.value) === String(k.default), `knob ${k.key} default value, got ${el && el.value}`);
}
for (const id of ['seed', 'run-btn', 'random-seed', 'batch-runs', 'batch-btn', 'knobs', 'knobs-reset', 'run-status']) {
  check($(id), 'missing #' + id);
}

if (!$('run-btn') || !$('seed') || !$('knob-boss_attempts')) done();

// Run with seed and an edited knob; buttons disabled while pending
mode = 'hold';
$('seed').value = '7';
$('knob-boss_attempts').value = '3';
$('run-btn').click();
await tick();
const runUrl = v.fetchCalls.find(u => u.startsWith('/api/run'));
check(runUrl && /[?&]seed=7(&|$)/.test(runUrl), 'run URL must contain seed=7, got ' + runUrl);
check(runUrl && /[?&]boss_attempts=3(&|$)/.test(runUrl), 'run URL must contain boss_attempts=3, got ' + runUrl);
check(runUrl && /[?&]monster_hp_mult=1(\.0+)?(&|$)/.test(runUrl), 'run URL sends all knobs (monster_hp_mult), got ' + runUrl);
check($('run-btn').disabled && $('batch-btn').disabled, 'buttons must be disabled while request pending');
check(typeof pendingRun === 'function', 'run request should be pending');
if (pendingRun) pendingRun();
await tick();
check(!$('run-btn').disabled && !$('batch-btn').disabled, 'buttons must be re-enabled after response');
check(String($('scrub').max) === '79', 'scrub max must be 79 after run, got ' + $('scrub').max);
check(JSON.stringify(v.win.__jv.logIndices()) === '[0]', 'log shows event 0 only after new run, got ' + JSON.stringify(v.win.__jv.logIndices().slice(0, 5)));
check(/seed 7/.test($('info').textContent), 'info shows seed 7, got ' + $('info').textContent);
