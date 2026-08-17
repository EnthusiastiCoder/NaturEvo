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
