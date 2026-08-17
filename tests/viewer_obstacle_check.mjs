// Obstacle types in the map viewer (plans/2026-10-21_hero-map-walking.md, Addendum 3 part A, viewer).
// Written first; fails until the feature exists. Stub-DOM harness conventions of viewer_map_check.mjs.
//
// Names picked where the plan is silent:
//   tile chars '#' Wall, 'R' Rock, 'F' Fence, 'T' Tree, 'W' Water (plus '.' ',' 'C'), /api/map adds `legend`.
//   tokens --tile-wall --tile-rock --tile-fence --tile-tree --tile-water in both themes (+ existing --tile-floor).
//   each blocking tile is painted with its token fill (a fillRect covering the tile) and then at least one more
//   drawing op inside that tile (the pattern: rock blob, fence rails/posts, canopy, wave stripes, brick lines).
//   legend swatches in #maplegend use classes sw-wall sw-rock sw-fence sw-tree sw-water sw-floor sw-grass sw-camp
//   (8 tile entries, in markup or built from MAP.legend; swatch classes must exist in the final DOM/markup).
//   tooltip (#maptip) text contains the kind name: Wall, Rock, Fence, Tree, Water.
import { html, load, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_obstacle_check');
const N = 40;
const KINDS = [['#', 'Wall', '--tile-wall'], ['R', 'Rock', '--tile-rock'], ['F', 'Fence', '--tile-fence'], ['T', 'Tree', '--tile-tree'], ['W', 'Water', '--tile-water']];
const row20 = '.'.repeat(4) + '#.R.F.T.W' + '.'.repeat(N - 13);
const tiles = Array.from({ length: N }, (_, y) => y === 20 ? row20 : '.'.repeat(N)).map((r, y) => y === 0 ? 'C' + r.slice(1) : r);
const zones = Array.from({ length: N }, () => '1'.repeat(N));
const legend = [['#', 'Wall', true], ['R', 'Rock', true], ['F', 'Fence', true], ['T', 'Tree', true], ['W', 'Water', true], ['.', 'Floor', false], [',', 'Grass', false], ['C', 'Camp', false]].map(([ch, name, blocking]) => ({ ch, name, blocking }));
const MAP = { w: N, h: N, tiles, zones, legend, anchors: Array.from({ length: 9 }, (_, i) => [i * 4, 5]) };
const hero = (x, y) => ({ level: 5, hp: 20, max_hp: 20, xp: 0, gold: 0, potions: 0, zone: 1, weapon: 0, armor: 0, x, y });
const RUN = { seed: 1, won: false, deaths: 0, final_level: 5, w: N, h: N, steps: 9, events: [0, 1, 2].map(i => ({ idx: i, kind: 'travel', text: 'e' + i, hero: hero(10, 10), mon: null, npcs: [] })) };
const norm = c => String(c || '').trim().toLowerCase().replace(/\s+/g, '');

function boot(map = MAP) {
  const rec = { ops: [] };
  const fetchImpl = async url => url.startsWith('/api/map') ? jsonResp(map) : jsonResp({ error: 'nf' }, 404);
  const v = load({ protocol: 'http:', run: RUN, fetchImpl, pre: w => { w.devicePixelRatio = 1; } });
  const cv = v.$('map');
  const st = { fillStyle: '', strokeStyle: '' };
  cv.getContext = () => new Proxy({}, {
    get: (t, k) => (k in t ? t[k] : (...a) => { rec.ops.push({ op: String(k), a, fill: norm(st.fillStyle), stroke: norm(st.strokeStyle) }); }),
    set: (t, k, val) => { t[k] = val; if (k === 'fillStyle' || k === 'strokeStyle') st[k] = val; return true; },
  });
  for (const e of [cv, cv.parent]) if (e) { e.clientWidth = 640; e.clientHeight = 480; }
  cv.getBoundingClientRect = () => ({ left: 0, top: 0, width: 640, height: 480, right: 640, bottom: 480 });
  const tok = n => String(v.win.getComputedStyle(v.root).getPropertyValue(n) || '').trim();
  return { v, cv, rec, tok };
}
const redraw = async (b, theme) => {
  if (theme) { b.v.root.dataset.theme = theme; b.v.win.dispatchEvent({ type: 'jv-themechange' }); }
  await tick(); b.rec.ops.length = 0; b.v.win.__jv.goto(1); await tick();
};
/** the tile rect painted with the kind token, and the signature of every other op whose point lies inside it */
function tileInfo(b, token) {
  const base = norm(b.tok(token));
  const r = b.rec.ops.find(o => o.op === 'fillRect' && o.fill === base && o.a[2] > 0 && Math.abs(o.a[2] - o.a[3]) < 1e-6);
  if (!r) return null;
  const [x, y, w, h] = r.a;
  const inside = o => o.a.length >= 2 && typeof o.a[0] === 'number' && o.a[0] >= x - 1e-6 && o.a[0] < x + w - 1e-6 && o.a[1] >= y - 1e-6 && o.a[1] < y + h - 1e-6;
  const extra = b.rec.ops.filter(o => o !== r && /^(fillRect|strokeRect|arc|ellipse|moveTo|lineTo|rect)$/.test(o.op) && inside(o));
  return { rect: r.a, sig: base + '|' + extra.map(o => `${o.op}:${o.fill}:${o.stroke}`).sort().join(','), extra };
}
