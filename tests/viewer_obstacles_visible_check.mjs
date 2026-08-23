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
