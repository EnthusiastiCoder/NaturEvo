// Map canvas requirements (plans/2026-10-21_hero-map-walking.md, workstream 3).
// Written from the plan before the implementation, using the stub-DOM harness (_viewer_harness.mjs).
//
// Names picked where the plan is silent (the implementation must use these, or this test is updated):
//   #map canvas, #mapzoom button (Zoom toggle), #maptip tooltip element (class "hidden" when not shown),
//   #mapempty element (empty state, class "hidden" when a map is shown),
//   localStorage "jv-mapzoom" = "1" (zoomed) / "0" (bird's-eye),
//   GET /api/map?seed=<n>&map_size=<n> is fetched over http: (JSON shape of the plan, section 9),
//   markers are drawn with fillRect/strokeRect/arc using fillStyle/strokeStyle = tok('--accent') (hero),
//   tok('--warn') (elite npc, id starting "e"), tok('--bad') (npc id "boss").
// Pixel positions are derived from the drawn markers themselves (the harness has no real pixels).
import { html, load, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_map_check');

// ---------------- fixtures ----------------
const N = 40;
const tiles = Array.from({ length: N }, (_, y) => Array.from({ length: N }, (_, x) => (x === 0 && y === 0 ? 'C' : y === N - 1 ? '#' : (x + y) % 3 === 0 ? ',' : '.')).join(''));
const zones = Array.from({ length: N }, () => Array.from({ length: N }, (_, x) => String(Math.min(8, Math.floor(x / 5)))).join(''));
const MAP = { w: N, h: N, tiles, zones, anchors: Array.from({ length: 9 }, (_, i) => [i * 5, 5]) };
const hero = (x, y, zone) => ({ level: 5, hp: 20, max_hp: 20, xp: 0, gold: 0, potions: 0, zone, weapon: 0, armor: 0, x, y });
const ev = (i, kind, h, extra = {}) => ({ idx: i, kind, text: kind + ' ' + i, hero: h, mon: null, npcs: [], ...extra });
const RUN = {
  seed: 1, won: false, deaths: 0, final_level: 5, w: N, h: N, steps: 99,
  events: [
    ev(0, 'travel', hero(10, 10, 2), { text: 'Hero begins the journey' }),
    ev(1, 'walk', hero(14, 12, 2), { text: 'Hero walks', path: [[10, 10], [10, 12], [14, 12]] }),
    ev(2, 'encounter', hero(14, 12, 2), { text: 'Hero meets a level 3 Vine Snake', mon: { species: 1, level: 3, hp: 5, max_hp: 5, x: 14, y: 12 },
      npcs: [{ id: 'e1', sp: 2, lv: 12, x: 17, y: 12 }, { id: 'boss', sp: 15, lv: 32, x: 12, y: 15 }] }),
    ev(3, 'walk', hero(20, 12, 4), { text: 'Hero walks again', path: [[14, 12], [20, 12]] }),
  ],
};
const OLD_RUN = { seed: 1, won: false, deaths: 0, final_level: 1, events: [0, 1, 2].map(i => ({ idx: i, kind: 'travel', text: 'event ' + i, hero: { level: 1, hp: 10, max_hp: 10, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0 }, mon: null })) };

const norm = c => String(c || '').trim().toLowerCase().replace(/\s+/g, '');
const centre = o => (o.op === 'arc' || o.op === 'ellipse') ? [o.a[0], o.a[1]] : [o.a[0] + o.a[2] / 2, o.a[1] + o.a[3] / 2];
const SHAPES = new Set(['fillRect', 'strokeRect', 'arc', 'ellipse']);

function boot({ dpr = 1, W = 640, H = 480, run = RUN, map = MAP, storage = {}, mapFetch = 'ok' } = {}) {
  const rec = { ops: [], smoothing: [] };
  const fetchImpl = async url => {
    if (url.startsWith('/api/map')) {
      if (mapFetch === 'ok') return jsonResp(map);
      if (mapFetch === '404') return jsonResp({ error: 'nf' }, 404);
      throw new Error('network down');
    }
    return jsonResp({ error: 'nf' }, 404);
  };
  const v = load({ protocol: 'http:', run, storage, fetchImpl, pre: w => { w.devicePixelRatio = dpr; } });
  const cv = v.$('map');
  let ctx;
  if (cv) {
    const st = { fillStyle: '', strokeStyle: '' };
    ctx = new Proxy({}, {
      get: (t, k) => (k in t ? t[k] : (...a) => { rec.ops.push({ op: String(k), a, fill: norm(st.fillStyle), stroke: norm(st.strokeStyle) }); }),
      set: (t, k, val) => { t[k] = val; if (k === 'fillStyle' || k === 'strokeStyle') st[k] = val; if (k === 'imageSmoothingEnabled') rec.smoothing.push(val); return true; },
    });
    cv.getContext = () => ctx;
    for (const e of [cv, cv.parent]) if (e) { e.clientWidth = W; e.clientHeight = H; }
    cv.getBoundingClientRect = () => ({ left: 0, top: 0, width: W, height: H, right: W, bottom: H });
  }
  const tok = n => String(v.win.getComputedStyle(v.root).getPropertyValue(n) || '').trim();
  return { v, cv, rec, ctx, tok, W, H, dpr };
}
const goto = async (b, i) => { b.rec.ops.length = 0; b.v.win.__jv.goto(i); await tick(); };
const marker = (b, colourToken) => b.rec.ops.filter(o => SHAPES.has(o.op) && (o.fill === norm(b.tok(colourToken)) || o.stroke === norm(b.tok(colourToken))));
/** pixel transform {ts, ox, oy} derived from hero markers at events 0 (10,10) and 1 (14,12); zoomed maps are not derivable this way */
async function transform(b) {
  await goto(b, 0);
  const a = marker(b, '--accent').map(centre)[0];
  await goto(b, 1);
  const c = marker(b, '--accent').map(centre)[0];
  if (!a || !c) return null;
  const ts = (c[0] - a[0]) / 4;
  return { ts, tsY: (c[1] - a[1]) / 2, ox: a[0] - 10.5 * ts, oy: a[1] - 10.5 * ts, a, c };
}
const px = (t, x, y) => [t.ox + (x + 0.5) * t.ts, t.oy + (y + 0.5) * t.ts];
const near = (p, q, tol) => Math.abs(p[0] - q[0]) <= tol && Math.abs(p[1] - q[1]) <= tol;
