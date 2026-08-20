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
// Timing (user feedback: too fast): a walk step takes 400 ms at 1x (readable), speed multiplies up to 8x, above that walk
// playback is capped at 8x and the info line says 'skipping steps'. Non-walk events stay 100 ms. The harness steps Date.now by 50 ms per interval call.
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
    h.play(); await h.pump(300); // first walk step lands at 300 ms (play banks 100 ms); sample at each tick after it
    const tiles = [], seen = [];
    for (let i = 0; i < 12; i++) { await h.pump(400); tiles.push(h.anim().tile && h.anim().tile.slice()); seen.push(h.anim().step); }
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
    h.play(); await h.pump(9000); h.play();
    const idx = h.log().filter(e => e[0] === 1).map(e => e[1]);
    check(eq(idx, STEPS.map((_, k) => k)), `full walk: step log for the walk must be 0..19, got ${JSON.stringify(idx)}`);
    check(eq(h.anim().tile, h.anim().tile) && h.jv().stepLog.length >= NSTEP, 'full walk: log must hold every tick');
  }
}

// ---------- 4x: four times as many steps per second, still one tile per sub-tick ----------
{
  const count = async s => {
    const h = boot(); h.jv().goto(0); h.setSpeed(s);
    if (!hasApi(h, s + 'x')) return null;
    h.play(); await h.pump(300); const a = h.log().length; await h.pump(1600); const n = h.log().length - a;
    const idx = h.log().filter(e => e[0] === 1).map(e => e[1]);
    idx.forEach((v, i) => { if (i && v !== idx[i - 1] + 1) check(false, `${s}x: sub-tick skipped a step ${idx[i - 1]} -> ${v}`); });
    return n;
  };
  const one = await count(1), four = await count(4);
  if (one !== null) check(four === 4 * one || Math.abs(four - 4 * one) <= 1, `4x must take 4 times as many steps per second as 1x (${four} vs ${one})`);
}

// ---------- glide: strictly between two adjacent tile centres up to 8x; snap at 16x ----------
{
  const sample = async speed => {
    const h = boot(); h.jv().goto(0); h.setSpeed(speed);
    if (!hasApi(h, speed + 'x glide')) return null;
    h.play(); await h.pump(300);
    const out = [];
    for (let i = 0; i < 6; i++) {
      // advance a frame inside the tick period (tick = 100/speed ms; frames are 16 ms) without firing the interval
      await h.advance(16);
      out.push({ pos: (h.anim().pos || []).slice(), tile: (h.anim().tile || []).slice(), walking: h.anim().step >= 0 });
      await h.pump(400 / Math.min(speed, 8));
    }
    return out;
  };
  for (const sp of [1, 4, 8]) {
    const s = await sample(sp);
    if (!s) continue;
    const mids = s.filter(o => o.walking);
    check(mids.length >= 3, `${sp}x: walk must be in progress during sampling`);
    for (const o of mids) {
      const [x, y] = o.pos, between = !isInt(x) && isInt(y) && Math.abs(x - o.tile[0]) < 1 && Math.abs(x - o.tile[0]) > 0 && Math.abs(Math.round(x - 0.5 + 0.5) - x) > 0;
      const lo = Math.floor(x), hi = Math.ceil(x);
      check(between && hi - lo === 1 && Math.abs(o.tile[0] - lo) <= 1 && o.tile[1] === y, `${sp}x: mid-tick drawn position ${JSON.stringify(o.pos)} must lie strictly between two adjacent tile centres next to tile ${JSON.stringify(o.tile)}`);
    }
  }
  // above 8x walk playback is capped at 8x: still a glide, still one tile per tick (see the cap block below)
  const s16 = await sample(16);
  if (s16) for (const o of s16.filter(o => o.walking)) check(Math.abs(o.pos[0] - o.tile[0]) <= 1 + 1e-9 && o.pos[1] === o.tile[1], `16x: drawn ${JSON.stringify(o.pos)} must stay within one tile of ${JSON.stringify(o.tile)}`);
}

// ---------- elites move one tile per tick, in sync with the hero ----------
{
  const h = boot(); h.jv().goto(0); h.setSpeed(1);
  if (hasApi(h, 'elite')) {
    h.play(); await h.pump(300);
    let prev = null;
    for (let i = 0; i < 10; i++) {
      await h.pump(400);
      const k = h.anim().step, nt = (h.anim().ntile || {}).e1;
      check(k >= 0 && eq(nt, ELITE[k]), `elite: committed tile ${JSON.stringify(nt)} must equal nsteps[${k}] ${JSON.stringify(ELITE[k])}`);
      if (prev && nt) check(manh(prev, nt) === 1, `elite: moved ${JSON.stringify(prev)} -> ${JSON.stringify(nt)}, expected one tile per tick`);
      prev = nt;
    }
    // elite glides too: mid-tick drawn position between two tile centres
    await h.advance(16);
    const np = (h.anim().npcs || {}).e1;
    check(np && isInt(np[1]) && !isInt(np[0]), `elite: mid-tick drawn position ${JSON.stringify(np)} must be between two tile centres`);
  }
}

// ---------- pause freezes, play resumes from the same step ----------
{
  const h = boot(); h.jv().goto(0); h.setSpeed(1);
  if (hasApi(h, 'pause')) {
    h.play(); await h.pump(2000);
    h.play(); await h.pump(50); await h.advance(500);
    const k = h.anim().step, tile = h.anim().tile.slice(), n = h.log().length, pos = h.anim().pos.slice();
    check(k > 0 && k < NSTEP - 1, `pause: must be mid-walk, step ${k}`);
    await h.pump(2000);
    check(h.log().length === n && eq(h.anim().tile, tile), 'pause: step must not advance while paused');
    check(eq(h.anim().pos, pos) || isInt(h.anim().pos[0]), `pause: position must hold, ${JSON.stringify(pos)} -> ${JSON.stringify(h.anim().pos)}`);
    h.play(); await h.pump(500);
    const after = h.log().slice(n);
    check(after.length >= 1 && after[0][0] === 1 && after[0][1] === k + 1, `pause: play must resume at step ${k + 1}, got ${JSON.stringify(after[0])}`);
  }
}

// ---------- scrubbing and seeking ----------
{
  const h = boot();
  if (hasApi(h, 'scrub')) {
    for (const k of [9, 0, 19, 3, 15, 3]) {
      h.jv().gotoStep(1, k); await h.advance(1000);
      check(eq(h.anim().tile, STEPS[k]) && eq(h.anim().pos, STEPS[k]), `gotoStep(1,${k}): tile and drawn position must be ${JSON.stringify(STEPS[k])}, got ${JSON.stringify(h.anim().tile)} / ${JSON.stringify(h.anim().pos)}`);
      check(h.anim().step === k, `gotoStep(1,${k}): step index must be ${k}, got ${h.anim().step}`);
      check(eq(h.anim().ntile && h.anim().ntile.e1, ELITE[k]), `gotoStep(1,${k}): elite must be at ${JSON.stringify(ELITE[k])}`);
    }
    // seeking to the event shows the end tile; back and forth is consistent
    for (const i of [1, 0, 2, 1, 150, 1, 0]) {
      h.jv().goto(i); await h.advance(1000);
      const want = i === 0 ? [10, 10] : i === 1 ? [30, 10] : [30, 10];
      check(eq(h.anim().pos, want), `goto(${i}): hero must be at ${JSON.stringify(want)}, got ${JSON.stringify(h.anim().pos)}`);
    }
    check(h.errors.length === 0, 'scrub: no errors: ' + h.errors.map(String));
  }
}

// ---------- windowed fetch across a walk boundary ----------
{
  const total = 6000, run = mkRun(300); run.total = total;
  const fetchImpl = async url => {
    const from = +/from=(\d+)/.exec(url)[1], count = +/count=(\d+)/.exec(url)[1];
    const r = mkRun(Math.min(count, total - from)); r.from = from; r.total = total;
    r.events.forEach((e, i) => { e.idx = from + i; });
    return jsonResp(r);
  };
  // walk event as the last event of the first window
  run.events[299].kind = 'walk'; run.events[299].steps = STEPS.map(s => s.slice()); run.events[299].nsteps = ELITE.map(([x, y]) => [['e1', x, y]]);
  run.events[299].path = [[10, 10], [30, 10]]; run.events[299].hero.x = 30;
  run.events[1].kind = 'travel'; delete run.events[1].steps; delete run.events[1].nsteps; delete run.events[1].path;
  const h = boot({ run, protocol: 'http:', fetchImpl });
  if (hasApi(h, 'window')) {
    h.jv().goto(298); h.setSpeed(1);
    try { h.play(); await h.pump(12000); h.play(); } catch (e) { h.errors.push(e); }
    check(h.errors.length === 0, 'window: crossing a walk boundary must not throw: ' + h.errors.map(String));
    const idx = h.log().filter(e => e[0] === 299).map(e => e[1]);
    check(eq(idx, STEPS.map((_, k) => k)), `window: no steps skipped across the boundary, got ${JSON.stringify(idx)}`);
  }
}

// ---------- crisp canvas, no scroll, both themes, no errors ----------
for (const theme of ['dark', 'light']) {
  const h = boot(); h.v.root.dataset.theme = theme; h.v.win.dispatchEvent({ type: 'jv-themechange' });
  h.jv().goto(0); h.setSpeed(1);
  if (hasApi(h, theme)) {
    h.play(); await h.pump(300); await h.advance(32);
    check(h.map.width === 640 && h.map.height === 640, `${theme}: canvas backing size must equal CSS size at dpr 1 mid-glide, got ${h.map.width}x${h.map.height}`);
    check(h.errors.length === 0, `${theme}: no errors mid-walk: ` + h.errors.map(String));
  }
}
{
  const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
  check(!/overflow\s*:\s*scroll/i.test(markup), 'no forced scroll');
  check(/imageSmoothingEnabled\s*=\s*false/.test(html), 'map must keep image smoothing off');
}

// ---------- old-format data without steps: path fallback or snap, no exception ----------
{
  for (const [label, mod] of [['path only', r => r], ['no path no steps', r => { delete r.events[1].path; return r; }]]) {
    const h = boot({ run: mod(mkRun(300, { withSteps: false })) });
    h.jv().goto(0); h.setSpeed(1);
    try { h.play(); await h.pump(4000); h.play(); h.jv().goto(1); await h.advance(1000); h.jv().goto(0); await h.advance(100); } catch (e) { h.errors.push(e); }
    check(h.errors.length === 0, `old format (${label}): must not throw: ` + h.errors.map(String));
    h.jv().goto(1); await h.advance(1000);
    check(eq(h.anim().pos, [30, 10]), `old format (${label}): hero must end at (30,10), got ${JSON.stringify(h.anim().pos)}`);
    if (typeof h.jv().gotoStep === 'function') { try { h.jv().gotoStep(1, 5); } catch (e) { check(false, `old format (${label}): gotoStep must not throw: ${e}`); } }
  }
}

done();
