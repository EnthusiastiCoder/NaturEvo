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
