import { load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_postgame_check');
const hidden = el => !!el && el.classList.contains('hidden');
const shown = el => !!el && !hidden(el) && el.style.display !== 'none';
