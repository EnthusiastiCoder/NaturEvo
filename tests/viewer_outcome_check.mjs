// Outcome badge, dashboard death stats and replay stop (plans/2026-10-21_hero-map-walking.md, Addendum 4 part 1, viewer).
// Written first, failing. Stub-DOM harness conventions (_viewer_harness.mjs).
//
// Names picked where the plan is silent:
//   run head field `outcome` is "won" | "died" | "capped"; badge text "Victory" | "Fallen" | "Capped".
//   The existing #outcome badge in the summary strip carries the text when the replay is at the final event
//   (before that it must not show an outcome word). Old runs without `outcome` fall back to the `won` flag:
//   won -> Victory, not won and capped -> Capped, otherwise Fallen. No exception for missing fields.
//   Dashboard (#dstats) shows death stats from STATS.died / died_pct: text matches /fallen|died|death/i and
//   contains the died_pct with one decimal and a percent sign (e.g. "62.5%").
//   Replay stop: the final event of a died run is kind "end" after kind "death"; play stops there (play button
//   aria-label back to "Play", progress stays at the last index when the interval fires again).
import { load, tick, runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_outcome_check');
const sleep = ms => new Promise(r => setTimeout(r, ms));
const hero = (hp = 10) => ({ level: 5, hp, max_hp: 10, xp: 0, gold: 0, potions: 0, zone: 1, weapon: 0, armor: 0, x: 3, y: 3 });
const ev = (i, kind, hp) => ({ idx: i, kind, text: kind + ' ' + i, hero: hero(hp), mon: null, npcs: [] });
const mk = (outcome, extra = {}) => {
  const kinds = outcome === 'died' ? ['travel', 'travel', 'death', 'end'] : ['travel', 'travel', 'travel', 'travel'];
  const events = kinds.map((k, i) => ev(i, k, k === 'death' || k === 'end' ? 0 : 10));
  const run = { seed: 1, won: outcome === 'won', deaths: outcome === 'died' ? 1 : 0, final_level: 5, steps: 0, total: events.length, events, ...extra };
  if (outcome !== undefined) run.outcome = outcome;
  return run;
};
const boot = async run => { const v = load({ protocol: 'file:', run }); await tick(); return v; };
const badge = v => String(v.$('outcome') ? v.$('outcome').textContent : '').trim();

// ---- badge text per outcome, at the last event
for (const [outcome, word] of [['won', 'Victory'], ['died', 'Fallen'], ['capped', 'Capped']]) {
  const run = mk(outcome, outcome === 'capped' ? { capped: true } : {});
  const v = await boot(run);
  v.win.__jv.goto(run.events.length - 1); await tick();
  check(badge(v) === word, `${outcome}: summary badge at the last event must read "${word}", got ${JSON.stringify(badge(v))}`);
  v.win.__jv.goto(0); await tick();
  check(!/victory|fallen|capped/i.test(badge(v)), `${outcome}: badge must not show an outcome before the final event, got ${JSON.stringify(badge(v))}`);
}
