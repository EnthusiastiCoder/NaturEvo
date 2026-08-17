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

function boot(opts = {}) {
  const errors = [];
  let now = 1e6; const raf = [];
  const pre = w => {
    w.devicePixelRatio = 1; w.innerWidth = 1280; w.innerHeight = 720;
    w.ResizeObserver = class { observe() {} unobserve() {} disconnect() {} };
    w.requestAnimationFrame = f => { raf.push(f); return raf.length; };
    w.cancelAnimationFrame = () => {};
    w.performance = { now: () => now };
    const RD = Date; w.Date = class extends RD { static now() { return now; } };
  };
  const v = load({ run: opts.run || mkRun(), pre, protocol: opts.protocol || 'file:', fetchImpl: opts.fetchImpl });
  const map = v.$('map');
  if (map) { map.clientWidth = 640; map.clientHeight = 640; if (map.parent) { map.parent.clientWidth = 640; map.parent.clientHeight = 640; } }
  const frames = () => { const q = raf.splice(0); q.forEach(f => { try { f(now); } catch (e) { errors.push(e); } }); };
  const advance = async ms => { for (let t = 0; t < ms; t += 16) { now += 16; frames(); } await tick(); };
  const pump = async ms => { for (let t = 0; t < ms; t += 50) { now += 50; v.intervals.forEach(f => { try { f(); } catch (e) { errors.push(e); } }); frames(); } await tick(); };
  const anim = () => v.win.__mapAnim || {};
  const jv = () => v.win.__jv;
  const setSpeed = s => { const n = v.$('speed-num'); n.value = String(s); n.fire('change'); };
  const log = () => (jv().stepLog || []).map(e => e.slice());
  const play = () => v.$('play').click();
  return { v, map, advance, pump, anim, jv, setSpeed, log, play, errors, raf };
}
const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const isInt = n => Math.abs(n - Math.round(n)) < 1e-9;
const manh = (a, b) => Math.abs(a[0] - b[0]) + Math.abs(a[1] - b[1]);
const hasApi = (h, msg) => { const ok = typeof h.jv().gotoStep === 'function' && Array.isArray(h.jv().stepLog); check(ok, msg + ': __jv.gotoStep and __jv.stepLog must exist'); return ok; };

// ---------- 1x: one tile per tick over the 20-tile walk ----------
{
  const h = boot(); h.jv().goto(0); h.setSpeed(1);
  if (hasApi(h, '1x')) {
    h.play(); await h.pump(100); // first interval call carries one extra tick; sample after it
    const tiles = [], seen = [];
    for (let i = 0; i < 12; i++) { await h.pump(100); tiles.push(h.anim().tile && h.anim().tile.slice()); seen.push(h.anim().step); }
    const walkSteps = h.log().filter(e => e[0] === 1);
    check(walkSteps.length >= 12, `1x: expected the walk to be stepping, log has ${walkSteps.length} walk steps`);
    walkSteps.forEach((e, i) => { if (i && e[1] !== walkSteps[i - 1][1] + 1) check(false, `1x: step index jumped ${walkSteps[i - 1][1]} -> ${e[1]}`); });
    for (let i = 1; i < tiles.length; i++) check(tiles[i] && tiles[i - 1] && manh(tiles[i], tiles[i - 1]) === 1, `1x: tick ${i} moved ${JSON.stringify(tiles[i - 1])} -> ${JSON.stringify(tiles[i])}, expected exactly one tile`);
    for (let i = 0; i < tiles.length; i++) check(eq(tiles[i], STEPS[seen[i]]), `1x: committed tile ${JSON.stringify(tiles[i])} must be steps[${seen[i]}] ${JSON.stringify(STEPS[seen[i]])}`);
    // drawn tile sequence equals `steps` (every step, in order, nothing skipped)
    const idx = walkSteps.map(e => e[1]);
    check(eq(idx, idx.map((_, i) => idx[0] + i)), 'drawn tile sequence must be consecutive entries of steps');
    check(h.errors.length === 0, '1x: no errors: ' + h.errors.map(String));
  }
}

// ---------- the whole walk is shown: step log is exactly 0..19 for the walk event ----------
{
  const h = boot(); h.jv().goto(0); h.setSpeed(1);
  if (hasApi(h, 'full walk')) {
    h.play(); await h.pump(4000); h.play();
    const idx = h.log().filter(e => e[0] === 1).map(e => e[1]);
    check(eq(idx, STEPS.map((_, k) => k)), `full walk: step log for the walk must be 0..19, got ${JSON.stringify(idx)}`);
    check(eq(h.anim().tile, h.anim().tile) && h.jv().stepLog.length >= NSTEP, 'full walk: log must hold every tick');
  }
}
