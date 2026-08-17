// Replay on the map + shared attack animation (plans/2026-10-21_hero-map-walking.md, "Addendum: replay on the
// map and shared attack animation"). Written first; fails until the feature exists.
// Observable contract used here (from the plan): window.__mapAnim = {active, kind, from:[x,y], to:[x,y], t}.
// ASSUMPTION (plan does not name a marker hook): __mapAnim also carries `pos:[x,y]`, the hero marker's current
// tile position (fractional while animating), valid whenever the map has drawn a frame.
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_replay_map_check');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
check(/<canvas\b[^>]*\bid\s*=\s*["']map["']/.test(markup), '#map canvas must exist');

// ---- run with map data: x,y on hero, mon, walk paths, attack kinds ----
function mapRun(n) {
  const r = makeRun(n);
  r.total = n;
  r.events.forEach((e, i) => {
    e.hero.x = 10 + (i % 20); e.hero.y = 5 + Math.floor(i / 20);
    e.npcs = [{ id: 'e1', sp: 0, lv: 1, x: 30, y: 30 }];
  });
  const kinds = { 3: 'attack', 4: 'skill', 5: 'potion', 6: 'hit_taken' };
  for (const [i, k] of Object.entries(kinds)) {
    const e = r.events[i]; e.kind = k; e.mon = { name: 'x', hp: 5, max_hp: 5, x: e.hero.x + 1, y: e.hero.y };
  }
  const w = r.events[10]; w.kind = 'walk'; w.hero.x = 20; w.hero.y = 10; w.path = [[10, 10], [20, 10]];
  const w2 = r.events[11]; w2.kind = 'walk'; w2.hero.x = 20; w2.hero.y = 14; w2.path = [[20, 10], [20, 14]];
  return r;
}

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
  const origErr = console.error;
  const v = load({ run: opts.run || mapRun(300), pre, protocol: opts.protocol || 'file:', fetchImpl: opts.fetchImpl });
  const map = v.$('map');
  if (map) { map.clientWidth = 640; map.clientHeight = 640; if (map.parent) { map.parent.clientWidth = 640; map.parent.clientHeight = 640; } }
  const advance = async ms => { // advance fake clock in 16 ms frames, firing rAF callbacks and the play interval
    for (let t = 0; t < ms; t += 16) { now += 16; const q = raf.splice(0); q.forEach(f => f(now)); }
    await tick();
  };
  const anim = () => v.win.__mapAnim;
  const pos = () => (anim() && anim().pos) || null;
  const go = i => v.win.__jv.goto(i);
  const setSpeed = s => { const n = v.$('speed-num'); n.value = String(s); n.fire('change'); };
  const pump = async ms => { // play interval ticks
    for (let t = 0; t < ms; t += 50) { now += 50; v.intervals.forEach(f => { try { f(); } catch (e) { errors.push(e); } }); const q = raf.splice(0); q.forEach(f => f(now)); }
    await tick();
  };
  return { v, map, advance, anim, pos, go, setSpeed, pump, errors, raf };
}
const near = (p, x, y, eps = 1e-6) => p && Math.abs(p[0] - x) < eps && Math.abs(p[1] - y) < eps;
const evOf = (r, i) => r.events[i];

// ---------- seek moves the marker ----------
{
  const run = mapRun(300), h = boot({ run });
  h.v.$('tab-replay').click();
  for (const i of [0, 7, 150, 42, 150, 0, 299]) {
    h.go(i); await h.advance(1000);
    const e = evOf(run, i);
    check(near(h.pos(), e.hero.x, e.hero.y), `seek ${i}: marker must be at hero (${e.hero.x},${e.hero.y}), got ${JSON.stringify(h.pos())}`);
  }
}
