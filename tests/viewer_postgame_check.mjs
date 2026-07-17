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

// ---- stage badge
{
  const events = [ev(0, { mon: mon(0) }), ev(1, { mon: mon(3) }), ev(2, { mon: mon(1) }), ev(3)];
  const v = load({ protocol: 'file:', run: { seed: 1, events, won: true, deaths: 0, final_level: 40 } });
  await tick();
  const st = v.$('stage');
  check(st, '#stage element missing');
  if (st) {
    v.win.__jv.goto(1);
    check(shown(st) && /Stage 3/.test(st.textContent), 'stage 3 monster: #stage shows "Stage 3", got ' + JSON.stringify(st.textContent) + ' hidden=' + hidden(st));
    v.win.__jv.goto(2);
    check(/Stage 1/.test(st.textContent), 'stage 1 text, got ' + JSON.stringify(st.textContent));
    v.win.__jv.goto(0);
    check(hidden(st) || st.textContent.trim() === '', 'stage 0: #stage must be hidden/empty, got ' + JSON.stringify(st.textContent));
    v.win.__jv.goto(1);
    v.win.__jv.goto(3);
    check(hidden(st) || st.textContent.trim() === '', 'no monster: #stage hidden/empty');
  }
}

// ---- old-format run (no total / level_series / stage) still renders
{
  let threw = null, v;
  try { v = load({ protocol: 'file:', run: makeRun(300) }); await tick(); v.win.__jv.goto(150); v.win.__jv.goto(299); } catch (e) { threw = e; }
  check(!threw, 'old-format run threw: ' + (threw && threw.message));
  if (v) {
    check(String(v.$('scrub').max) === '299', 'old run scrub max 299, got ' + v.$('scrub').max);
    check(v.fetchCalls.length === 0, 'old file:// run must not fetch');
    check(/299/.test(v.$('info').textContent), 'info shows event 299');
  }
}

// ---- windowed mode fetches from/count when goto leaves the window
{
  const TOTAL = 65536, W = 2000;
  const window_ = (from, count) => {
    const events = [];
    for (let i = from; i < Math.min(TOTAL, from + count); i++) events.push(ev(i));
    return { seed: 1, won: true, deaths: 0, final_level: 40, total: TOTAL, from, events, level_series: Array.from({ length: 500 }, (_, i) => 1 + Math.floor(i / 10)) };
  };
  const fetchImpl = async (url) => {
    if (url.startsWith('/api/run')) {
      const f = +(url.match(/[?&]from=(\d+)/) || [])[1], c = +(url.match(/[?&]count=(\d+)/) || [])[1];
      return jsonResp(window_(f, c));
    }
    return jsonResp({ error: 'nf' }, 404);
  };
  const v = load({ protocol: 'http:', run: window_(0, W), fetchImpl });
  await tick();
  v.fetchCalls.length = 0;
  const jv = v.win.__jv;
  jv.goto(100);
  await tick();
  check(!v.fetchCalls.some(u => /from=/.test(u)), 'goto inside loaded window must not fetch, got ' + JSON.stringify(v.fetchCalls));
  try { jv.goto(40000); } catch (e) { check(false, 'goto outside loaded window threw: ' + e.message); }
  await tick();
  const u = v.fetchCalls.find(x => x.startsWith('/api/run'));
  check(u && /[?&]from=\d+/.test(u) && /[?&]count=\d+/.test(u), 'leaving window must fetch /api/run?...from=&count=, got ' + JSON.stringify(v.fetchCalls));
  check(u && /[?&]seed=1(&|$)/.test(u), 'window fetch keeps seed=1, got ' + u);
  if (u) {
    const from = +u.match(/from=(\d+)/)[1], count = +u.match(/count=(\d+)/)[1];
    check(from <= 40000 && 40000 < from + count && count <= 5000, `window [${from},${from + count}) must contain 40000 and count<=5000`);
  }
  let idx = [];
  try { idx = jv.logIndices(); } catch (e) {}
  check(idx.length > 0 && idx[idx.length - 1] === 40000, 'after fetch the log ends at global event 40000, got ' + JSON.stringify(idx.slice(-3)));
  check(String(v.$('scrub').max) === String(TOTAL - 1), 'scrub max spans total (65535), got ' + v.$('scrub').max);
}
