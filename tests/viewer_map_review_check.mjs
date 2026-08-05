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

// In-page helpers (device-pixel image access, token colours).
const HELP = `(() => {
  const cv = () => document.getElementById('map');
  const data = () => { const c = cv(); return c.getContext('2d').getImageData(0, 0, c.width, c.height); };
  const tokRgb = n => { const v = getComputedStyle(document.documentElement).getPropertyValue(n).trim(); const m = v.match(/^#([0-9a-f]{6})$/i); return m ? [0,2,4].map(i => parseInt(m[1].slice(i, i+2), 16)) : null; };
  window.__t = {
    geom() { const c = cv(); const r = c.getBoundingClientRect(); return { W: c.width, H: c.height, cssW: r.width, cssH: r.height, dpr: window.devicePixelRatio }; },
    px(x, y) { const c = cv(); const d = c.getContext('2d').getImageData(Math.round(x), Math.round(y), 1, 1).data; return [d[0], d[1], d[2], d[3]]; },
    tok: tokRgb,
    filled() { const d = data().data; let n = 0; for (let i = 3; i < d.length; i += 4) if (d[i] > 0) n++; return n / (d.length / 4); },
    // bbox of pixels within tol (euclid rgb) of rgb inside box [x0,y0,x1,y1] (device px)
    near(rgb, tol, box) { const im = data(), W = im.width, d = im.data; let a = 1e9, b = 1e9, c = -1, e = -1, n = 0;
      for (let y = Math.max(0, Math.floor(box[1])); y < Math.min(im.height, Math.ceil(box[3])); y++) for (let x = Math.max(0, Math.floor(box[0])); x < Math.min(W, Math.ceil(box[2])); x++) {
        const i = (y * W + x) * 4; if (d[i+3] < 200) continue; const dd = Math.hypot(d[i]-rgb[0], d[i+1]-rgb[1], d[i+2]-rgb[2]);
        if (dd <= tol) { n++; if (x < a) a = x; if (x > c) c = x; if (y < b) b = y; if (y > e) e = y; } }
      return { n, x0: a, y0: b, x1: c, y1: e, w: c - a + 1, h: e - b + 1 }; },
    save(name) { window.__f = window.__f || {}; window.__f[name] = data().data.slice(); },
    diff(name, box) { const im = data(), W = im.width, d = im.data, o = window.__f[name]; let n = 0;
      for (let y = Math.max(0, Math.floor(box[1])); y < Math.min(im.height, Math.ceil(box[3])); y++) for (let x = Math.max(0, Math.floor(box[0])); x < Math.min(W, Math.ceil(box[2])); x++) {
        const i = (y * W + x) * 4; if (Math.abs(d[i]-o[i]) + Math.abs(d[i+1]-o[i+1]) + Math.abs(d[i+2]-o[i+2]) > 12) n++; } return n; },
  };
  return true; })()`;

async function open(url, { w, h, dpr = 1, theme = null, pre = null, helpers = true } = {}) {
  const { browserContextId } = await send('Target.createBrowserContext');
  const { targetId } = await send('Target.createTarget', { url: 'about:blank', browserContextId });
  const { sessionId } = await send('Target.attachToTarget', { targetId, flatten: true });
  const S = (m, p) => send(m, p, sessionId);
  const log = { errors: [], bad: [], reqs: new Map() };
  const onEv = d => {
    if (d.sessionId !== sessionId) return;
    if (d.method === 'Runtime.exceptionThrown') log.errors.push('exception: ' + (d.params.exceptionDetails.exception && d.params.exceptionDetails.exception.description || d.params.exceptionDetails.text));
    if (d.method === 'Runtime.consoleAPICalled' && d.params.type === 'error') log.errors.push('console.error: ' + d.params.args.map(a => a.value || a.description).join(' '));
    if (d.method === 'Log.entryAdded' && d.params.entry.level === 'error') log.errors.push('log: ' + d.params.entry.text + ' ' + (d.params.entry.url || ''));
    if (d.method === 'Network.requestWillBeSent') log.reqs.set(d.params.requestId, d.params.request.url);
    if (d.method === 'Network.responseReceived' && d.params.response.status >= 400) log.bad.push(d.params.response.status + ' ' + d.params.response.url);
    if (d.method === 'Network.loadingFailed' && !d.params.canceled) log.bad.push('failed ' + (log.reqs.get(d.params.requestId) || '') + ' ' + d.params.errorText);
  };
  handlers.push(onEv);
  for (const m of ['Page.enable', 'Runtime.enable', 'Network.enable', 'Log.enable']) await S(m);
  await S('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: dpr, mobile: false });
  if (theme) await S('Page.addScriptToEvaluateOnNewDocument', { source: `try{localStorage.setItem('jv-theme','${theme}')}catch(e){}` });
  if (pre) await S('Page.addScriptToEvaluateOnNewDocument', { source: pre });
  await S('Page.navigate', { url });
  const page = {
    log, S,
    ev: async expr => { const r = await S('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true }); if (r.exceptionDetails) throw new Error(expr.slice(0, 80) + ' -> ' + (r.exceptionDetails.exception && r.exceptionDetails.exception.description)); return r.result.value; },
    close: async () => { handlers.splice(handlers.indexOf(onEv), 1); try { await send('Target.disposeBrowserContext', { browserContextId }); } catch { /* */ } },
  };
  if (helpers) { for (let i = 0; i < 100; i++) { await sleep(50); if (await page.ev('!!document.getElementById("map") && document.readyState!=="loading"').catch(() => false)) break; } await page.ev(HELP).catch(() => {}); }
  return page;
}
const settle = async (p, idx) => { await p.ev(`window.__jv.goto(${idx})`); await sleep(600); };
