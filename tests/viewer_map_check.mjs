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

// ---------------- static: markup and tokens ----------------
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
const css = (html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1].replace(/\/\*[\s\S]*?\*\//g, '');
{
  const tag = (markup.match(/<canvas\b[^>]*\bid\s*=\s*["']map["'][^>]*>/i) || [])[0];
  check(tag, '<canvas id="map"> missing in markup');
  if (tag) {
    check(!/\swidth\s*=|\sheight\s*=/i.test(tag), '#map must not have width/height attributes');
    check(!/\bstyle\s*=/i.test(tag), '#map must not have inline style');
  }
  const iMap = markup.search(/id\s*=\s*["']map["']/), iZone = markup.search(/id\s*=\s*["']zone["']/);
  check(iMap >= 0 && iZone >= 0 && iMap < iZone, '#map must come before #zone in the centre column');
  check(/id\s*=\s*["']mapzoom["']/.test(markup), '#mapzoom Zoom toggle button missing');
  check(/id\s*=\s*["']maptip["']/.test(markup), '#maptip tooltip element missing');
  check(/id\s*=\s*["']mapempty["']/.test(markup), '#mapempty empty-state element missing');
  const idCss = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].filter(m => m[1].split(',').some(s => s.trim().endsWith('#map')));
  for (const m of idCss) for (const d of m[2].split(';')) {
    const mm = d.match(/^\s*(width|height)\s*:\s*([^;]+)$/i);
    if (mm) check(!/^\s*\d+(\.\d+)?px\s*$/i.test(mm[2]), `#map css must not set fixed-px ${mm[1]}`);
  }
}
{
  const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(m => ({ sels: m[1].split(',').map(s => s.trim().replace(/\s+/g, ' ')), body: m[2] }));
  const tokens = t => Object.fromEntries(rules.filter(r => r.sels.some(s => new RegExp('^:root\\[data-theme=["\']?' + t + '["\']?\\]$').test(s))).flatMap(r => r.body.split(';').map(d => { const k = d.indexOf(':'); return k < 0 ? null : [d.slice(0, k).trim(), d.slice(k + 1).trim()]; }).filter(Boolean)));
  const dark = tokens('dark'), light = tokens('light');
  const NEED = ['--tile-floor', '--tile-grass', '--tile-wall', '--tile-camp', ...Array.from({ length: 9 }, (_, i) => '--biome-' + i)];
  for (const n of NEED) {
    check(dark[n], `token ${n} missing in the dark theme block`);
    check(light[n], `token ${n} missing in the light theme block`);
  }
  for (const n of ['--tile-floor', '--tile-grass', '--tile-wall', '--tile-camp']) check(dark[n] && light[n] && norm(dark[n]) !== norm(light[n]), `${n} must differ between dark and light`);
  // contrast of the hero marker (accent) against the tiles, both themes (>= 3:1)
  const rgb = c => { const m = String(c || '').trim().match(/^#([0-9a-f]{6})$/i); return m ? [0, 2, 4].map(i => parseInt(m[1].slice(i, i + 2), 16)) : null; };
  const lum = c => { const [r, g, b] = rgb(c).map(x => { x /= 255; return x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
  const cr = (a, b) => (Math.max(lum(a), lum(b)) + 0.05) / (Math.min(lum(a), lum(b)) + 0.05);
  const accent = (rules.find(r => r.sels.includes(':root')).body.match(/--accent\s*:\s*([^;]+)/) || [])[1];
  for (const [name, set] of [['dark', dark], ['light', light]]) for (const n of ['--tile-floor', '--tile-grass']) {
    if (rgb(accent) && rgb(set[n])) check(cr(accent, set[n]) >= 3, `hero marker (accent ${accent}) vs ${n} in ${name}: contrast ${cr(accent, set[n]).toFixed(2)} < 3`);
    else check(false, `cannot compute contrast for ${n} in ${name} (tokens must be #rrggbb)`);
  }
}

// ---------------- behaviour ----------------
for (const [W, H, vw, vh] of [[640, 480, 1280, 720], [1000, 700, 1920, 1080], [380, 300, 800, 600]]) {
  for (const dpr of [1, 2]) {
    const label = `${vw}x${vh} dpr ${dpr}`;
    const b = boot({ dpr, W, H });
    await tick();
    if (!b.cv) { check(false, '#map canvas missing at ' + label); continue; }
    check(b.v.fetchCalls.some(u => u.startsWith('/api/map')), label + ': map must be fetched from /api/map');
    await goto(b, 0);
    check(b.cv.width === Math.round(b.cv.clientWidth * dpr) && b.cv.height === Math.round(b.cv.clientHeight * dpr), `${label}: backing size ${b.cv.width}x${b.cv.height} != css ${b.cv.clientWidth}x${b.cv.clientHeight} * dpr`);
    check(b.cv.clientWidth <= vw && b.cv.clientHeight <= vh, label + ': canvas larger than the viewport');
    check(b.rec.smoothing.length > 0 && b.rec.smoothing.every(x => x === false), label + ': imageSmoothingEnabled must be set to false');
    if (dpr === 1) {
      const t = await transform(b);
      if (!t) { check(false, label + ': no accent hero marker drawn'); continue; }
      check(Number.isInteger(t.ts) && t.ts >= 1, `${label}: bird's-eye tile size must be a positive integer, got ${t.ts}`);
      check(Math.abs(t.tsY - t.ts) < 1e-6, label + ': tiles must be square');
      check(t.ts * N <= Math.min(W, H) + 1e-6, `${label}: whole map must fit: ${t.ts}*${N} > ${Math.min(W, H)}`);
      check(t.ox >= -1e-6 && t.oy >= -1e-6 && t.ox + t.ts * N <= W + 1e-6 && t.oy + t.ts * N <= H + 1e-6, label + ': map extends beyond the canvas');
    }
  }
}

{
  const b = boot({ W: 640, H: 480 });
  await tick();
  const t = await transform(b);
  check(t, 'transform not derivable');
  if (t) {
    // hero marker is at the pixel of (x,y) and moves with the cursor
    check(near(t.a, px(t, 10, 10), t.ts / 2), 'hero marker not at tile (10,10)');
    check(near(t.c, px(t, 14, 12), t.ts / 2), 'hero marker did not move to tile (14,12)');
    check(Math.abs(t.c[0] - t.a[0]) > 0, 'hero marker must move when the cursor moves');
    // NPC markers come from event npcs
    await goto(b, 2);
    const el = marker(b, '--warn').map(centre), boss = marker(b, '--bad').map(centre);
    check(el.some(p => near(p, px(t, 17, 12), t.ts / 2)), 'elite npc e1 marker (warn colour) not at (17,12)');
    check(boss.some(p => near(p, px(t, 12, 15), t.ts / 2)), 'boss marker (bad colour) not at (12,15)');
    await goto(b, 1);
    check(!marker(b, '--warn').length, 'no elite marker expected when the event has no npcs');
    // hover tooltip
    const tip = b.v.$('maptip');
    await goto(b, 2);
    const hp = px(t, 7, 3);
    b.cv.fire('mousemove', { offsetX: hp[0], offsetY: hp[1], clientX: hp[0], clientY: hp[1] });
    const tx = tip ? tip.textContent : '';
    check(/\b7\s*,\s*3\b/.test(tx), 'tooltip must show the tile coordinates 7,3, got ' + JSON.stringify(tx));
    check(/Jungle Edge/.test(tx), 'tooltip must show the zone name (zone 1 = Jungle Edge), got ' + JSON.stringify(tx));
    check(tip && !tip.classList.contains('hidden'), 'tooltip must be visible while hovering');
    const np = px(t, 17, 12);
    b.cv.fire('mousemove', { offsetX: np[0], offsetY: np[1], clientX: np[0], clientY: np[1] });
    check(/e1|elite|Elite/.test(tip.textContent), 'tooltip over an npc must mention it, got ' + JSON.stringify(tip.textContent));
    b.cv.fire('mouseleave', {});
    check(tip.classList.contains('hidden') || tip.textContent.trim() === '', 'tooltip must hide on mouseleave');
    // grass vs floor kind in the tooltip
    const gp = px(t, 6, 3); // (6+3)%3===0 -> grass
    b.cv.fire('mousemove', { offsetX: gp[0], offsetY: gp[1], clientX: gp[0], clientY: gp[1] });
    check(/grass/i.test(tip.textContent), 'tooltip must show the tile kind (grass), got ' + JSON.stringify(tip.textContent));
    b.cv.fire('mouseleave', {});

    // zoom toggle, double click, persistence
    const tsBird = t.ts;
    const zoomBtn = b.v.$('mapzoom');
    check(zoomBtn, 'Zoom button missing');
    if (zoomBtn) {
      zoomBtn.click();
      await goto(b, 2);
      check(b.v.store.get('jv-mapzoom') === '1', 'zoom state must persist as jv-mapzoom=1, got ' + b.v.store.get('jv-mapzoom'));
      const h = marker(b, '--accent').map(centre)[0], e1 = marker(b, '--warn').map(centre)[0];
      check(h && Math.abs(h[0] - b.W / 2) <= b.W / 16 && Math.abs(h[1] - b.H / 2) <= b.H / 16, 'zoomed: hero marker must stay at the canvas centre, got ' + JSON.stringify(h));
      if (h && e1) {
        const tsZ = (e1[0] - h[0]) / 3;
        check(Number.isInteger(tsZ) && tsZ >= 2 * tsBird, `zoomed tile size ${tsZ} must be an integer >= 2x bird's-eye ${tsBird}`);
        check(tsZ * 16 <= Math.min(b.W, b.H) + 1e-6, `16x16 window must fit: ${tsZ}*16`);
      } else check(false, 'zoomed: elite npc within 3 tiles must still be drawn');
      await goto(b, 3);
      const h3 = marker(b, '--accent').map(centre)[0];
      check(h3 && Math.abs(h3[0] - b.W / 2) <= b.W / 16, 'zoomed: hero marker stays centred after the hero moves');
      zoomBtn.click();
      check(b.v.store.get('jv-mapzoom') === '0', 'toggling back must store jv-mapzoom=0');
      await goto(b, 1);
      const back = marker(b, '--accent').map(centre)[0];
      check(back && near(back, px(t, 14, 12), t.ts / 2), 'bird\'s-eye view must be restored');
    }
    b.cv.fire('dblclick', { offsetX: 100, offsetY: 100, clientX: 100, clientY: 100 });
    check(b.v.store.get('jv-mapzoom') === '1', 'double click must switch to zoom');
    b.cv.fire('dblclick', { offsetX: 100, offsetY: 100, clientX: 100, clientY: 100 });
    check(b.v.store.get('jv-mapzoom') === '0', 'double click again must switch back');
  }
  // persisted zoom is applied on load
  const z = boot({ W: 640, H: 480, storage: { 'jv-mapzoom': '1' } });
  await tick();
  await goto(z, 1);
  const h = marker(z, '--accent').map(centre)[0];
  check(h && Math.abs(h[0] - 320) <= 40 && Math.abs(h[1] - 240) <= 30, 'stored zoom must start zoomed (hero at the centre), got ' + JSON.stringify(h));
  // log lines show (x,y) for walk and encounter events
  const log = b.v.$('log');
  b.v.win.__jv.goto(3);
  const lt = (log.textContent || '') + (log.innerHTML || '');
  check(/\(\s*20\s*,\s*12\s*\)/.test(lt), 'log line for the walk event must show (20,12)');
  check(/\(\s*14\s*,\s*12\s*\)/.test(lt), 'log line for the encounter event must show (14,12)');
}

// ---------------- both themes: drawn tile colours come from tokens and differ ----------------
{
  const drawn = {};
  for (const theme of ['dark', 'light']) {
    const b = boot({ storage: { 'jv-theme': theme } });
    await tick();
    await goto(b, 2);
    const want = ['--tile-floor', '--tile-grass', '--tile-wall', '--tile-camp'].map(n => norm(b.tok(n)));
    drawn[theme] = want;
    check(want.every(Boolean), `${theme}: tile tokens must resolve`);
    const fills = new Set(b.rec.ops.filter(o => o.op === 'fillRect').map(o => o.fill));
    check(want.slice(0, 3).every(c => fills.has(c)), `${theme}: tile fills must use the --tile-* token values, drew ${[...fills].join(',')}`);
    check(marker(b, '--accent').length > 0, theme + ': hero marker missing');
  }
  check(drawn.dark.join() !== drawn.light.join(), 'tile colours must differ between themes');
}
