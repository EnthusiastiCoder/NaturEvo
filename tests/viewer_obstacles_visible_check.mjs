// Obstacles must be clearly visible on the 2D map (plans/2026-10-21_hero-map-walking.md, Addendum 4 part 3).
// Written first, failing. Real pixels: like viewer_map_review_check.mjs this drives a real headless Edge over the
// DevTools protocol (node's built-in WebSocket) against the real server, with the runner() conventions of
// _viewer_harness.mjs (check()/done(), "FAIL: ..." lines, exit code 1, no dependencies).
//
// Names/geometry used (the implementation must match or this test is updated):
//   #map canvas, #mapzoom (Zoom button), #theme-toggle (real toggle, clicked), #maplegend (children = legend entries;
//   swatch = a child element with class sw-wall sw-rock sw-fence sw-tree sw-water, either a <canvas> (centre pixel)
//   or an element with a CSS background-color), tokens --tile-floor --tile-grass --tile-rock --tile-fence --tile-tree
//   --tile-water --tile-wall in both themes. Tile chars from /api/map: '#' wall, 'R' rock, 'F' fence, 'T' tree,
//   'W' water, '.' floor, ',' grass, 'C' camp; zones are digit chars.
//   Bird's-eye geometry (device px): ts = floor(min(W,H)/mapW), ox = floor((W-ts*mapW)/2), oy likewise.
//   Zoom: 16x16 tile window, ts = W/16, top-left tile = clamp(hero - 8, 0, mapW - 16).
//   Contrast is WCAG relative-luminance contrast >= 3:1; kinds differ pairwise by RGB distance >= 25.
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, cpSync, readFileSync, rmSync } from 'node:fs';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_obstacles_visible_check');
const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const sleep = ms => new Promise(r => setTimeout(r, ms));

const EDGE = ['C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe', 'C:/Program Files/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Google/Chrome/Application/chrome.exe'].find(existsSync);
const cargoEnv = { ...process.env, PATH: join(process.env.USERPROFILE || '', '.cargo', 'bin') + ';/c/Users/enthu/.cargo/bin;' + process.env.PATH };
const exe = join(root, 'sim', 'target', 'release', 'jungle-sim.exe');
if (!EDGE) { check(false, 'no Edge/Chrome found for the browser checks'); done(); }
const build = spawnSync('cargo', ['build', '--release'], { cwd: join(root, 'sim'), env: cargoEnv, encoding: 'utf8', shell: true });
if (build.status !== 0) console.error('warning: cargo build failed, using the existing binary if any: ' + (build.stderr || '').slice(-300));
if (!existsSync(exe)) { check(false, 'sim binary missing: ' + exe); done(); }

const tmp = mkdtempSync(join(tmpdir(), 'jv-obst-'));
const site = join(tmp, 'site');
cpSync(join(root, 'viewer'), site, { recursive: true });
const gen = spawnSync(exe, ['run', '--seed', '1', '--out', join(site, 'data', 'run.js')], { encoding: 'utf8' });
if (gen.status !== 0) { check(false, 'sim run failed: ' + gen.stderr); done(); }
const RUN = JSON.parse(readFileSync(join(site, 'data', 'run.js'), 'utf8').replace(/^[^{]*/, '').replace(/;\s*$/, ''));
const MW = RUN.w || 64, EV = RUN.events;

const freePort = () => new Promise(res => { const s = createServer(); s.listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(() => res(p)); }); });
const port = await freePort();
const server = spawn(exe, ['serve', '--port', String(port), '--root', site], { stdio: 'ignore' });
const BASE = `http://127.0.0.1:${port}/`;

const udd = join(tmp, 'profile');
const edge = spawn(EDGE, ['--headless=new', '--remote-debugging-port=0', '--user-data-dir=' + udd, '--no-first-run', '--disable-gpu', '--hide-scrollbars', 'about:blank'], { stdio: 'ignore' });
function cleanup() { try { ws && ws.close(); } catch { /* */ } try { edge.kill(); } catch { /* */ } try { server.kill(); } catch { /* */ } try { rmSync(tmp, { recursive: true, force: true }); } catch { /* */ } }
let wsUrl = null, ws = null;
for (let i = 0; i < 100 && !wsUrl; i++) {
  await sleep(100);
  try { const [p, path] = readFileSync(join(udd, 'DevToolsActivePort'), 'utf8').trim().split('\n'); wsUrl = `ws://127.0.0.1:${p}${path}`; } catch { /* not yet */ }
}
if (!wsUrl) { check(false, 'could not start headless browser'); cleanup(); done(); }
ws = new WebSocket(wsUrl);
await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
let nid = 0; const pending = new Map();
ws.onmessage = m => {
  const d = JSON.parse(m.data);
  if (d.id && pending.has(d.id)) { const { res, rej } = pending.get(d.id); pending.delete(d.id); d.error ? rej(new Error(d.error.message)) : res(d.result); }
};
const send = (method, params = {}, sessionId) => new Promise((res, rej) => { const id = ++nid; pending.set(id, { res, rej }); ws.send(JSON.stringify({ id, method, params, sessionId })); });

const HELP = `(() => {
  const cv = () => document.getElementById('map');
  const rgbOf = s => { const m = String(s).match(/rgba?\\(([^)]+)\\)/); if (m) return m[1].split(',').slice(0, 3).map(x => Math.round(parseFloat(x))); const h = String(s).trim().match(/^#([0-9a-f]{6})$/i); return h ? [0,2,4].map(i => parseInt(h[1].slice(i, i+2), 16)) : null; };
  window.__t = {
    geom() { const c = cv(), r = c.getBoundingClientRect(); return { W: c.width, H: c.height, cssW: r.width, dpr: window.devicePixelRatio }; },
    px(x, y) { const d = cv().getContext('2d').getImageData(Math.round(x), Math.round(y), 1, 1).data; return [d[0], d[1], d[2]]; },
    tile(x0, y0, w, h) { const d = cv().getContext('2d').getImageData(Math.round(x0), Math.round(y0), Math.max(1, Math.round(w)), Math.max(1, Math.round(h))).data; const s = new Set(); for (let i = 0; i < d.length; i += 4) s.add((d[i] >> 3) + ',' + (d[i+1] >> 3) + ',' + (d[i+2] >> 3)); return s.size; },
    tok(n) { return rgbOf(getComputedStyle(document.documentElement).getPropertyValue(n)); },
    theme() { return document.documentElement.getAttribute('data-theme'); },
    swatches() { const l = document.getElementById('maplegend'); if (!l) return null; const out = {};
      for (const e of l.querySelectorAll('*')) { const m = String(e.className && e.className.baseVal !== undefined ? e.className.baseVal : e.className).match(/\\bsw-(wall|rock|fence|tree|water|floor|grass)\\b/); if (!m) continue;
        let rgb = null; const cvs = e.tagName === 'CANVAS' ? e : e.querySelector('canvas');
        if (cvs) { const d = cvs.getContext('2d').getImageData(Math.floor(cvs.width / 2), Math.floor(cvs.height / 2), 1, 1).data; rgb = [d[0], d[1], d[2]]; }
        else rgb = rgbOf(getComputedStyle(e).backgroundColor);
        out[m[1]] = rgb; }
      return out; },
  };
  return true; })()`;

async function open(url, { w, h, dpr = 1 }) {
  const { browserContextId } = await send('Target.createBrowserContext');
  const { targetId } = await send('Target.createTarget', { url: 'about:blank', browserContextId });
  const { sessionId } = await send('Target.attachToTarget', { targetId, flatten: true });
  const S = (m, p) => send(m, p, sessionId);
  for (const m of ['Page.enable', 'Runtime.enable']) await S(m);
  await S('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: dpr, mobile: false });
  await S('Page.navigate', { url });
  const page = {
    ev: async expr => { const r = await S('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true }); if (r.exceptionDetails) throw new Error(expr.slice(0, 80) + ' -> ' + (r.exceptionDetails.exception && r.exceptionDetails.exception.description)); return r.result.value; },
    close: async () => { try { await send('Target.disposeBrowserContext', { browserContextId }); } catch { /* */ } },
  };
  for (let i = 0; i < 100; i++) { await sleep(50); if (await page.ev('!!document.getElementById("map") && document.readyState!=="loading"').catch(() => false)) break; }
  await page.ev(HELP).catch(() => {});
  return page;
}

const lumC = c => { const [r, g, b] = c.map(x => { x /= 255; return x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
const contrast = (a, b) => (Math.max(lumC(a), lumC(b)) + 0.05) / (Math.min(lumC(a), lumC(b)) + 0.05);
const dist = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
const KINDS = { '#': 'wall', R: 'rock', F: 'fence', T: 'tree', W: 'water' };
const first = EV[0];
const near = (x, y, list, r) => list.some(n => Math.abs(n.x - x) + Math.abs(n.y - y) <= r);

try {
  const probe = await open(BASE, { w: 1282, h: 722 });
  await sleep(900);
  const MAP = await probe.ev(`fetch('/api/map?seed=1').then(r => r.json())`);
  await probe.close();
  check(MAP && MAP.tiles && MAP.tiles.length === MW, 'could not fetch /api/map?seed=1');
  const T = (x, y) => MAP.tiles[y][x], Z = (x, y) => MAP.zones[y][x];
  const avoid = [...(first.npcs || []), { x: first.hero.x, y: first.hero.y }];
  // a tile of each kind with no entity nearby and with at least one plain (floor/grass) 4-neighbour
  const pickKind = ch => { for (let y = 2; y < MW - 2; y++) for (let x = 2; x < MW - 2; x++) {
    if (T(x, y) !== ch || near(x, y, avoid, 4)) continue;
    const nb = [[1, 0], [-1, 0], [0, 1], [0, -1]].map(([dx, dy]) => [x + dx, y + dy]).find(([a, b]) => '.,'.includes(T(a, b)));
    if (nb) return { x, y, nb }; } return null; };
  const picks = {};
  for (const ch of Object.keys(KINDS)) { picks[ch] = pickKind(ch); check(picks[ch], `seed-1 map has no usable '${ch}' (${KINDS[ch]}) tile`); }
  const refTile = (ch, zone) => { for (let y = 2; y < MW - 2; y++) for (let x = 2; x < MW - 2; x++) if (T(x, y) === ch && Z(x, y) === zone && !near(x, y, avoid, 4)) return { x, y }; return null; };
  const share = MAP.tiles.join('').split('').filter(c => KINDS[c]).length / (MW * MW);
  check(share >= 0.05 && share <= 0.25, `obstacle tiles must be 5-25% of the map, got ${(share * 100).toFixed(1)}%`);

  const VIEWPORTS = [[1282, 642], [1282, 722], [1540, 890], [1920, 1080]];
  for (const [w, h] of VIEWPORTS) for (const dpr of [1, 2]) for (const theme of ['dark', 'light']) {
    const L = `${w}x${h} dpr ${dpr} ${theme}`;
    const p = await open(BASE, { w, h, dpr }); await sleep(900);
    for (let i = 0; i < 2 && await p.ev('__t.theme()') !== theme; i++) { await p.ev(`document.getElementById('theme-toggle').click()`); await sleep(300); }
    check(await p.ev('__t.theme()') === theme, `${L}: real theme toggle did not reach ${theme}`);
    await p.ev(`window.__jv.goto(${first.idx})`); await sleep(600);
    const g = await p.ev('__t.geom()');
    const ts = Math.floor(Math.min(g.W, g.H) / MW), ox = Math.floor((g.W - ts * MW) / 2), oy = Math.floor((g.H - ts * MW) / 2);
    check(ts / dpr >= 5, `${L}: bird's-eye tile must be >= 5 css px, got ${(ts / dpr).toFixed(2)}`);
    const centre = (x, y) => [ox + (x + 0.5) * ts, oy + (y + 0.5) * ts];
    const at = async (x, y) => { const c = centre(x, y); return p.ev(`__t.px(${c[0]},${c[1]})`); };
    const floorTok = await p.ev('__t.tok("--tile-floor")'), grassTok = await p.ev('__t.tok("--tile-grass")');
    const px = {};
    for (const [ch, name] of Object.entries(KINDS)) {
      const k = picks[ch]; if (!k) continue;
      px[name] = await at(k.x, k.y);
      const floorPx = (refTile('.', Z(k.x, k.y)) && await at(...Object.values(refTile('.', Z(k.x, k.y))))) || null;
      const grassPx = (refTile(',', Z(k.x, k.y)) && await at(...Object.values(refTile(',', Z(k.x, k.y))))) || null;
      const nbPx = await at(k.nb[0], k.nb[1]);
      check(dist(px[name], floorTok) >= 25, `${L}: ${name} centre pixel ${px[name]} must not be the floor colour ${floorTok}`);
      for (const [lab, ref] of [['floor pixel', floorPx], ['grass pixel', grassPx], ['neighbouring tint pixel', nbPx], ['floor token', floorTok], ['grass token', grassTok]]) {
        if (!ref) continue;
        check(contrast(px[name], ref) >= 3, `${L}: ${name} ${px[name]} vs ${lab} ${ref} contrast ${contrast(px[name], ref).toFixed(2)} must be >= 3:1`);
      }
    }
    const names = Object.keys(px);
    for (let i = 0; i < names.length; i++) for (let j = i + 1; j < names.length; j++)
      check(dist(px[names[i]], px[names[j]]) >= 25, `${L}: ${names[i]} ${px[names[i]]} and ${names[j]} ${px[names[j]]} differ by ${dist(px[names[i]], px[names[j]]).toFixed(1)} (< 25)`);

    // share of tiles whose centre pixel is neither the floor nor the grass look of its own zone
    const zoneRef = {};
    for (const z of new Set(MAP.zones.join('').split(''))) {
      const f = refTile('.', z), gr = refTile(',', z);
      zoneRef[z] = [f && await at(f.x, f.y), gr && await at(gr.x, gr.y)].filter(Boolean);
    }
    let other = 0, total = 0;
    for (let y = 1; y < MW - 1; y += 1) for (let x = 1; x < MW - 1; x += 3) {
      const ch = T(x, y); if (ch === 'C' || near(x, y, avoid, 3)) continue;
      const c = await at(x, y); total++;
      if (!(zoneRef[Z(x, y)] || []).some(r => dist(c, r) < 25)) other++;
    }
    check(other / total >= 0.05 && other / total <= 0.25, `${L}: non-floor/grass pixel share must be 5-25%, got ${(100 * other / total).toFixed(1)}%`);

    // legend swatches use the same colours as the map
    const sw = await p.ev('__t.swatches()');
    check(sw, `${L}: #maplegend missing`);
    if (sw) for (const name of Object.values(KINDS)) {
      check(sw[name], `${L}: legend swatch sw-${name} missing`);
      if (sw[name] && px[name]) check(dist(sw[name], px[name]) <= 20, `${L}: legend swatch ${name} ${sw[name]} must equal the map pixel ${px[name]}`);
    }
    await p.close();
  }
