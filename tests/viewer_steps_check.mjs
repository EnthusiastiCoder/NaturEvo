// One tile step per replay tick (plans/2026-10-21_hero-map-walking.md, Addendum 3 part B). Written first; fails
// until the feature exists. Conventions of viewer_replay_map_check.mjs (fake clock for Date/performance, manual
// play-interval and rAF pumping).
//
// Observable contract picked here (the plan names no hooks); the implementation must provide it:
//   walk events carry steps:[[x,y],...] (excl. start) and nsteps:[[[id,x,y],...],...] (one list per step).
//   window.__jv.stepLog   array; one entry [eventIdx, stepIdx] pushed for EVERY sub-cursor advance (every tick), in order.
//   window.__jv.gotoStep(eventIdx, stepIdx)  seeks into the middle of a walk (shows the hero on steps[stepIdx]).
//   window.__mapAnim.tile   [x,y] integer tile the hero has committed to (end tile of the current step);
//   window.__mapAnim.step   index of the current step inside a walk event, or -1 when not in a walk;
//   window.__mapAnim.ntile  {id:[x,y]} committed npc tiles; window.__mapAnim.npcs {id:[x,y]} drawn npc positions;
//   window.__mapAnim.pos    drawn hero position in tile units (tile centres are integers), fractional mid-glide.
// Timing: base 10 ticks per second at 1x; the harness steps Date.now by 50 ms per interval call.
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_steps_check');
const NSTEP = 20;
const STEPS = Array.from({ length: NSTEP }, (_, k) => [11 + k, 10]);        // hero walks east from (10,10)
const ELITE = Array.from({ length: NSTEP }, (_, k) => [40 - k, 12]);        // elite walks west one tile per hero step
const mkRun = (n = 300, { withSteps = true } = {}) => {
  const r = makeRun(n); r.total = n; r.w = 64; r.h = 64;
  r.events.forEach(e => { e.hero.x = 10; e.hero.y = 10; e.npcs = [{ id: 'e1', sp: 0, lv: 1, x: 40, y: 12 }]; });
  const w = r.events[1]; w.kind = 'walk'; w.hero.x = 30; w.hero.y = 10; w.path = [[10, 10], [30, 10]];
  w.npcs = [{ id: 'e1', sp: 0, lv: 1, x: 21, y: 12 }];
  if (withSteps) { w.steps = STEPS.map(s => s.slice()); w.nsteps = ELITE.map(([x, y], k) => [['e1', x, y]]); }
  for (let i = 2; i < n; i++) { r.events[i].hero.x = 30; r.events[i].hero.y = 10; r.events[i].npcs = [{ id: 'e1', sp: 0, lv: 1, x: 21, y: 12 }]; }
  return r;
};
