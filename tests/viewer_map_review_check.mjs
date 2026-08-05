// Map review fixes (plans/2026-10-21_hero-map-walking.md, "Addendum 2"). Written first, failing.
//
// The stub-DOM harness (_viewer_harness.mjs) has no layout engine and no canvas pixels, and these
// requirements are about real pixels and real boxes. So this file keeps the harness conventions
// (runner(): check()/done(), "FAIL: ..." lines, exit code 1 on failure, no dependencies) but drives a
// real headless Edge over the DevTools protocol (node's built-in WebSocket) against the real server:
//   - rebuilds sim (cargo build --release) and generates a seed-1 run into a temp copy of viewer/
//   - serves that copy with `jungle-sim serve --root <copy>` (so a missing route is a real 404)
//   - each scenario runs in an isolated browser context (fresh localStorage), with an exact
//     viewport and device pixel ratio.
//
// Names the implementation must use where the plan is silent (or this test is updated):
//   #map canvas inside #replay (existing), #mapzoom (existing Zoom button), #theme-toggle (existing),
//   #log-open (existing Event log button), #timeline (existing canvas),
//   #maplegend: legend container with exactly 6 entries (direct children), whose text names
//     hero, elite, boss, grass, wall, camp (any case),
//   window.__mapAnimStep(t): renders the attack frame at t in [0,1] with no timers; afterwards
//     window.__mapAnim.pos (hero) and .mpos (monster) are the drawn positions in tile units,
//   window.__mapAnim.active is true synchronously after __jv.goto(i) onto an attack/skill/potion/
//     hit_taken event with a mon (speed 1x), kind is the same string for every attack.
// Pixel geometry (device px, canvas W x H, square): bird's-eye tile ts = floor(min(W,H)/mapW),
//   ox = floor((W-ts*mapW)/2); zoom: 16x16 tile window, ts = W/16, hero tile centred.
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, cpSync, readFileSync, rmSync } from 'node:fs';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { pathToFileURL } from 'node:url';
import { runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_map_review_check');
const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const sleep = ms => new Promise(r => setTimeout(r, ms));

// ---------------- environment ----------------
const EDGE = ['C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe', 'C:/Program Files/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Google/Chrome/Application/chrome.exe'].find(existsSync);
const cargoEnv = { ...process.env, PATH: join(process.env.USERPROFILE || '', '.cargo', 'bin') + ';/c/Users/enthu/.cargo/bin;' + process.env.PATH };
const exe = join(root, 'sim', 'target', 'release', 'jungle-sim.exe');
if (!EDGE) { check(false, 'no Edge/Chrome found for the browser checks'); done(); }
const build = spawnSync('cargo', ['build', '--release'], { cwd: join(root, 'sim'), env: cargoEnv, encoding: 'utf8', shell: true });
if (build.status !== 0) console.error('warning: cargo build failed, using the existing binary if any: ' + (build.stderr || '').slice(-300));
if (!existsSync(exe)) { check(false, 'sim binary missing: ' + exe); done(); }

const tmp = mkdtempSync(join(tmpdir(), 'jv-review-'));
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

// ---------------- minimal CDP client ----------------
const udd = join(tmp, 'profile');
const edge = spawn(EDGE, ['--headless=new', '--remote-debugging-port=0', '--user-data-dir=' + udd, '--no-first-run', '--disable-gpu', '--hide-scrollbars', 'about:blank'], { stdio: 'ignore' });
let wsUrl = null;
for (let i = 0; i < 100 && !wsUrl; i++) {
  await sleep(100);
  try { const [p, path] = readFileSync(join(udd, 'DevToolsActivePort'), 'utf8').trim().split('\n'); wsUrl = `ws://127.0.0.1:${p}${path}`; } catch { /* not yet */ }
}
if (!wsUrl) { check(false, 'could not start headless browser'); cleanup(); done(); }
const ws = new WebSocket(wsUrl);
await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
let nid = 0; const pending = new Map(), handlers = [];
ws.onmessage = m => {
  const d = JSON.parse(m.data);
  if (d.id && pending.has(d.id)) { const { res, rej } = pending.get(d.id); pending.delete(d.id); d.error ? rej(new Error(d.error.message)) : res(d.result); }
  else if (d.method) handlers.forEach(h => h(d));
};
const send = (method, params = {}, sessionId) => new Promise((res, rej) => { const id = ++nid; pending.set(id, { res, rej }); ws.send(JSON.stringify({ id, method, params, sessionId })); });
function cleanup() { try { ws && ws.close(); } catch { /* */ } try { edge.kill(); } catch { /* */ } try { server.kill(); } catch { /* */ } try { rmSync(tmp, { recursive: true, force: true }); } catch { /* */ } }
