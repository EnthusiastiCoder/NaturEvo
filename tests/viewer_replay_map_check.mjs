// Replay on the map + shared attack animation (plans/2026-10-21_hero-map-walking.md, "Addendum: replay on the
// map and shared attack animation"). Written first; fails until the feature exists.
// Observable contract used here (from the plan): window.__mapAnim = {active, kind, from:[x,y], to:[x,y], t}.
// ASSUMPTION (plan does not name a marker hook): __mapAnim also carries `pos:[x,y]`, the hero marker's current
// tile position (fractional while animating), valid whenever the map has drawn a frame.
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_replay_map_check');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
check(/<canvas\b[^>]*\bid\s*=\s*["']map["']/.test(markup), '#map canvas must exist');
