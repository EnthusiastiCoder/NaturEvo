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
