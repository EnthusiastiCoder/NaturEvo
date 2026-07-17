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
