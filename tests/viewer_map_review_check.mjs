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

// ---------------- helpers ----------------
const lumC = c => { const [r, g, b] = c.map(x => { x /= 255; return x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
const contrast = (a, b) => (Math.max(lumC(a), lumC(b)) + 0.05) / (Math.min(lumC(a), lumC(b)) + 0.05);
const dist = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
const birdMap = g => { const ts = Math.floor(Math.min(g.W, g.H) / MW); const ox = Math.floor((g.W - ts * MW) / 2), oy = Math.floor((g.H - ts * MW) / 2); return { ts, c: (x, y) => [ox + (x + 0.5) * ts, oy + (y + 0.5) * ts] }; };
const inB = (v, lo = 10, hi = MW - 11) => v >= lo && v <= hi;
const heroOf = e => e.hero, away = e => inB(e.hero.x) && inB(e.hero.y);
const pickElite = EV.find(e => away(e) && (e.npcs || []).some(n => /^e/.test(n.id) && inB(n.x) && inB(n.y) && Math.max(Math.abs(n.x - e.hero.x), Math.abs(n.y - e.hero.y)) >= 3 && Math.max(Math.abs(n.x - e.hero.x), Math.abs(n.y - e.hero.y)) <= 6));
const pickBoss = EV.find(e => (e.npcs || []).some(n => n.id === 'boss' && inB(n.x, 4, MW - 5) && inB(n.y, 4, MW - 5)));
const pickAtk = EV.find(e => ['attack', 'skill', 'potion'].includes(e.kind) && e.mon && away(e) && inB(e.mon.x) && inB(e.mon.y));
const pickHit = EV.find(e => e.kind === 'hit_taken' && e.mon && away(e) && inB(e.mon.x) && inB(e.mon.y));
const pickFree = EV.find(e => away(e) && !(e.npcs || []).length && !e.mon);
const pickEdge = EV[0];
check(pickElite && pickBoss && pickAtk && pickHit && pickFree, 'fixture events missing in the seed-1 run: ' + JSON.stringify({ elite: !!pickElite, boss: !!pickBoss, atk: !!pickAtk, hit: !!pickHit, free: !!pickFree }));
const VIEWPORTS = [[1282, 642], [1282, 722], [1540, 890]];

try {
  // ============ 1. map size and squareness ============
  for (const [w, h] of VIEWPORTS) for (const dpr of [1, 2]) {
    const L = `${w}x${h} dpr ${dpr}`;
    const p = await open(BASE, { w, h, dpr }); await sleep(900);
    const m = await p.ev(`(() => { const r = document.getElementById('map').getBoundingClientRect(), v = document.getElementById('replay').getBoundingClientRect(), c = document.getElementById('map'); return { mw: r.width, mh: r.height, vw: v.width, vh: v.height, bw: c.width, bh: c.height }; })()`);
    const short = Math.min(m.mw, m.mh), vshort = Math.min(m.vw, m.vh);
    check(short >= 0.7 * vshort, `${L}: map short side ${short.toFixed(0)} must be >= 70% of #replay short side ${vshort.toFixed(0)} (>= ${(0.7 * vshort).toFixed(0)})`);
    check(Math.abs(m.mw - m.mh) <= 1, `${L}: map canvas must be square within 1px, got ${m.mw.toFixed(1)}x${m.mh.toFixed(1)}`);
    check(Math.abs(m.bw - m.mw * dpr) <= 1 && Math.abs(m.bh - m.mh * dpr) <= 1, `${L}: backing ${m.bw}x${m.bh} must be css*dpr`);
    if (dpr === 1) {
      const ts = Math.floor(Math.min(m.bw, m.bh) / MW);
      check(ts >= 4, `${L}: bird's-eye tile must be an integer >= 4 css px, got ${ts}`);
    }
    await p.close();
  }

  // ============ 2. markers and legend (both themes) ============
  for (const theme of ['dark', 'light']) {
    const L = theme + ' 1540x890';
    const p = await open(BASE, { w: 1540, h: 890, dpr: 1, theme }); await sleep(900);
    if (pickElite && pickFree && pickBoss) {
      const accent = await p.ev('__t.tok("--accent")'), warn = await p.ev('__t.tok("--warn")'), bad = await p.ev('__t.tok("--bad")');
      await settle(p, pickFree.idx);
      let g = await p.ev('__t.geom()'), bm = birdMap(g);
      const hc = bm.c(pickFree.hero.x, pickFree.hero.y), r = bm.ts * 4;
      const hb = await p.ev(`__t.near(${JSON.stringify(accent)}, 60, ${JSON.stringify([hc[0] - r, hc[1] - r, hc[0] + r, hc[1] + r])})`);
      check(hb.n > 0 && hb.w >= 8, `${L}: hero marker (accent) must be >= 8 css px across, got ${hb.n ? hb.w : 'none drawn'}`);
      check(hb.n > 0 && hb.w >= 1.5 * bm.ts, `${L}: hero marker must be >= 1.5 tiles across (${1.5 * bm.ts}), got ${hb.w}`);
      const heroCentre = await p.ev(`__t.px(${hc[0]},${hc[1]})`);
      await settle(p, pickElite.idx);
      g = await p.ev('__t.geom()'); bm = birdMap(g);
      const en = pickElite.npcs.find(n => /^e/.test(n.id)), ec = bm.c(en.x, en.y);
      const ep = await p.ev(`__t.px(${ec[0]},${ec[1]})`);
      const eb = await p.ev(`__t.near(${JSON.stringify(warn)}, 60, ${JSON.stringify([ec[0] - bm.ts * 2, ec[1] - bm.ts * 2, ec[0] + bm.ts * 2, ec[1] + bm.ts * 2])})`);
      check(eb.n > 0 && eb.w >= 8, `${L}: elite marker (warn colour) must be >= 8 css px across, got ${eb.n ? eb.w : 'none drawn'}`);
      check(dist(ep, heroCentre) > 40, `${L}: elite centre pixel ${ep} must differ from hero centre pixel ${heroCentre}`);
      check(contrast(ep.slice(0, 3), heroCentre.slice(0, 3)) >= 3, `${L}: hero vs elite fill contrast ${contrast(ep.slice(0, 3), heroCentre.slice(0, 3)).toFixed(2)} must be >= 3:1`);
      // hero ring vs tile: the ring colour (accent) against a plain tile pixel
      const tilePx = await p.ev(`__t.px(${hc[0] + bm.ts * 3},${hc[1] + bm.ts * 3})`);
      check(contrast(accent, tilePx.slice(0, 3)) >= 3, `${L}: hero ring (accent) vs tile contrast ${contrast(accent, tilePx.slice(0, 3)).toFixed(2)} must be >= 3:1`);
      await settle(p, pickBoss.idx);
      g = await p.ev('__t.geom()'); bm = birdMap(g);
      const bn = pickBoss.npcs.find(n => n.id === 'boss'), bc = bm.c(bn.x, bn.y);
      const bb = await p.ev(`__t.near(${JSON.stringify(bad)}, 60, ${JSON.stringify([bc[0] - bm.ts * 3, bc[1] - bm.ts * 3, bc[0] + bm.ts * 3, bc[1] + bm.ts * 3])})`);
      check(bb.n > 0 && bb.w >= 8, `${L}: boss marker (bad colour) must be >= 8 css px across, got ${bb.n ? bb.w : 'none drawn'}`);
      check(bb.n > 0 && eb.n > 0 && bb.w > eb.w, `${L}: boss marker (${bb.w}) must be larger than an elite marker (${eb.w})`);
    }
    const leg = await p.ev(`(() => { const l = document.getElementById('maplegend'); if (!l) return null; const r = l.getBoundingClientRect(); return { n: l.children.length, t: l.textContent.toLowerCase(), w: r.width, h: r.height }; })()`);
    check(leg, `${L}: #maplegend element missing`);
    if (leg) {
      check(leg.n === 6, `${L}: legend must have 6 entries, got ${leg.n}`);
      check(['hero', 'elite', 'boss', 'grass', 'wall', 'camp'].every(k => leg.t.includes(k)), `${L}: legend must name hero, elite, boss, grass, wall, camp, got ${JSON.stringify(leg.t.trim())}`);
      check(leg.w > 0 && leg.h > 0, `${L}: legend must be visible`);
    }
    await p.close();
  }

  // ============ 3. zoom ============
  for (const [w, h] of VIEWPORTS) {
    const L = `zoom ${w}x${h}`;
    const p = await open(BASE, { w, h, dpr: 1 }); await sleep(900);
    await p.ev(`document.getElementById('mapzoom') && document.getElementById('mapzoom').click()`);
    if (pickElite) {
      await settle(p, pickElite.idx);
      const g = await p.ev('__t.geom()'), ts = g.W / 16;
      check(g.cssW / 16 >= 24, `${L}: zoom tile must be >= 24 css px (16 tiles across), canvas ${g.cssW.toFixed(0)} -> ${(g.cssW / 16).toFixed(1)}`);
      const accent = await p.ev('__t.tok("--accent")');
      const hb = await p.ev(`__t.near(${JSON.stringify(accent)}, 60, ${JSON.stringify([g.W / 2 - ts * 1.5, g.H / 2 - ts * 1.5, g.W / 2 + ts * 1.5, g.H / 2 + ts * 1.5])})`);
      check(hb.n > 0 && Math.abs((hb.x0 + hb.x1) / 2 - g.W / 2) <= ts / 2 && Math.abs((hb.y0 + hb.y1) / 2 - g.H / 2) <= ts / 2, `${L}: hero must be at the canvas centre, accent pixels bbox ${JSON.stringify(hb)}`);
      const en = pickElite.npcs.find(n => /^e/.test(n.id));
      const nx = g.W / 2 + (en.x - pickElite.hero.x) * ts, ny = g.H / 2 + (en.y - pickElite.hero.y) * ts;
      const npc = await p.ev(`(() => { const a = __t.near(${JSON.stringify(await p.ev('__t.tok("--warn")'))}, 70, ${JSON.stringify([nx - ts / 2, ny - ts / 2, nx + ts / 2, ny + ts / 2])}); const c = __t.px(${nx},${ny}), t = __t.px(${nx - ts / 2 + 2},${ny - ts / 2 + 2}); return { a, c, t }; })()`);
      check(npc.a.n > 0 || dist(npc.c, npc.t) > 30, `${L}: NPC in the window must be drawn at its tile, centre pixel ${npc.c} vs tile pixel ${npc.t}`);
      const glyphs = await p.ev(`(() => { const im = document.getElementById('map').getContext('2d').getImageData(${Math.round(nx - ts / 2)}, ${Math.round(ny - ts / 2)}, ${Math.round(ts)}, ${Math.round(ts)}).data; const set = new Set(); for (let i = 0; i < im.length; i += 4) set.add((im[i] >> 4) + ',' + (im[i+1] >> 4) + ',' + (im[i+2] >> 4)); return set.size; })()`);
      check(glyphs >= 6, `${L}: NPC tile must show a species emoji (many colours), got ${glyphs} distinct colours`);
    }
    if (pickEdge) {
      await settle(p, pickEdge.idx);
      const g = await p.ev('__t.geom()'), ts = g.W / 16, accent = await p.ev('__t.tok("--accent")');
      const hb = await p.ev(`__t.near(${JSON.stringify(accent)}, 60, ${JSON.stringify([0, 0, g.W, g.H])})`);
      check(hb.n > 0 && (hb.x0 + hb.x1) / 2 < g.W / 2 - ts && (hb.y0 + hb.y1) / 2 < g.H / 2 - ts, `${L}: at the map corner the window is clamped, so the hero must be off centre, bbox ${JSON.stringify(hb)}`);
    }
    await p.close();
  }

  // ============ 4. first load, no interaction ============
  {
    const p = await open(BASE, { w: 1282, h: 722, dpr: 1 });
    let frac = 0; const t0 = Date.now();
    while (Date.now() - t0 < 3000 && frac <= 0.3) { await sleep(150); frac = await p.ev('(window.__t||(' + HELP + ')) && __t.filled()').catch(() => 0); }
    check(frac > 0.3, `http first load: map must be drawn within 3 s with no interaction, filled fraction ${(+frac).toFixed(2)}`);
    await sleep(500);
    check(p.log.bad.length === 0, 'http first load: no request may fail or 404, got ' + JSON.stringify(p.log.bad));
    check(p.log.errors.length === 0, 'http first load: no console errors, got ' + JSON.stringify(p.log.errors));
    await p.close();
    // run start / window refetch must not blank the canvas
    const q = await open(BASE, { w: 1282, h: 722, dpr: 1 }); await sleep(1200);
    const blank = await q.ev(`(async () => { let min = 1; const f = () => { try { min = Math.min(min, __t.filled()); } catch (e) { min = 0; } };
      for (let i = 0; i < 12; i++) { window.__jv.goto(i * 1100); f(); await new Promise(r => setTimeout(r, 120)); f(); } return min; })()`);
    check(blank > 0.3, `http: the canvas must never blank while seeking across windows, min filled ${(+blank).toFixed(2)}`);
    await q.close();
  }
  {
    const url = pathToFileURL(join(site, 'index.html')).href;
    const p = await open(url, { w: 1282, h: 722, dpr: 1 });
    let frac = 0; const t0 = Date.now();
    while (Date.now() - t0 < 3000 && frac <= 0.3) { await sleep(150); frac = await p.ev('(window.__t||(' + HELP + ')) && __t.filled()').catch(() => 0); }
    check(frac > 0.3, `file:// first load: map must be drawn within 3 s (window.MAP shipped in the run data), filled fraction ${(+frac).toFixed(2)}`);
    check(p.log.errors.length === 0, 'file:// first load: no console errors, got ' + JSON.stringify(p.log.errors));
    await p.close();
  }

  // ============ 5. attack animation hook ============
  {
    const p = await open(BASE, { w: 1540, h: 890, dpr: 1 }); await sleep(900);
    const kinds = {};
    for (const [label, e, who] of [['hero attack', pickAtk, 'hero'], ['hit_taken', pickHit, 'npc']]) {
      if (!e) continue;
      await settle(p, Math.max(0, e.idx - 1));
      const act = await p.ev(`(() => { window.__jv.goto(${e.idx}); const a = window.__mapAnim; return { active: a.active, kind: a.kind }; })()`);
      check(act.active === true, `${label}: __mapAnim.active must be true right after seeking to the event, got ${JSON.stringify(act)}`);
      kinds[label] = act.kind;
      await sleep(700);
      check(await p.ev('typeof window.__mapAnimStep') === 'function', `${label}: window.__mapAnimStep(t) hook missing`);
      if (await p.ev('typeof window.__mapAnimStep') !== 'function') continue;
      const g = await p.ev('__t.geom()'), bm = birdMap(g);
      await p.ev('window.__mapAnimStep(1)');
      const idle = await p.ev('({ pos: window.__mapAnim.pos.slice(), mpos: window.__mapAnim.mpos.slice() })');
      await p.ev('__t.save("idle")');
      await p.ev('window.__mapAnimStep(0.5)');
      const mid = await p.ev('({ pos: window.__mapAnim.pos.slice(), mpos: window.__mapAnim.mpos.slice() })');
      const tgt = who === 'hero' ? e.mon : e.hero, tc = bm.c(tgt.x, tgt.y), r = bm.ts * 1.5;
      const nd = await p.ev(`__t.diff("idle", ${JSON.stringify([tc[0] - r, tc[1] - r, tc[0] + r, tc[1] + r])})`);
      check(nd > 0, `${label}: pixels at the target tile must differ from the idle frame at t=0.5 (flash), changed pixels ${nd}`);
      const [a0, a1, t1] = who === 'hero' ? [idle.pos, mid.pos, idle.mpos] : [idle.mpos, mid.mpos, idle.pos];
      const toward = (t1[0] - a0[0]) * (a1[0] - a0[0]) + (t1[1] - a0[1]) * (a1[1] - a0[1]);
      check(Math.hypot(a1[0] - a0[0], a1[1] - a0[1]) > 0.05 && toward > 0, `${label}: attacker marker must be offset toward the target at t=0.5, idle ${JSON.stringify(a0)} mid ${JSON.stringify(a1)} target ${JSON.stringify(t1)}`);
    }
    if (kinds['hero attack'] && kinds['hit_taken']) check(kinds['hero attack'] === kinds['hit_taken'], `both directions must use the same animation kind, got ${JSON.stringify(kinds)}`);
    await p.close();
  }

  // ============ 6. layout: no scroll, no overlap ============
  for (const [w, h] of VIEWPORTS) {
    const L = `layout ${w}x${h}`;
    const p = await open(BASE, { w, h, dpr: 1 }); await sleep(1000);
    const m = await p.ev(`(() => {
      const R = e => { const r = e.getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height, r: r.right, b: r.bottom }; };
      const main = document.querySelector('.main'), de = document.documentElement, bd = document.body;
      const hit = (a, b) => Math.min(a.r, b.r) - Math.max(a.x, b.x) > 1 && Math.min(a.b, b.b) - Math.max(a.y, b.y) > 1;
      const logb = document.getElementById('log-open'), tl = document.getElementById('timeline');
      const vis = e => { const r = e.getBoundingClientRect(), s = getComputedStyle(e); return r.width > 0 && r.height > 0 && s.visibility !== 'hidden' && s.display !== 'none'; };
      const ctl = [...document.querySelectorAll('button, input, select, summary, a[href]')].filter(e => vis(e) && !e.closest('details:not([open])'));
      const bad = [];
      for (let i = 0; i < ctl.length; i++) for (let j = i + 1; j < ctl.length; j++) { if (ctl[i].contains(ctl[j]) || ctl[j].contains(ctl[i])) continue; if (hit(R(ctl[i]), R(ctl[j]))) bad.push((ctl[i].id || ctl[i].className || ctl[i].tagName) + ' x ' + (ctl[j].id || ctl[j].className || ctl[j].tagName)); }
      const cv = document.getElementById('map');
      return { main: main ? [main.scrollHeight, main.clientHeight] : null, doc: [de.scrollHeight, de.clientHeight, de.scrollWidth, de.clientWidth, bd.scrollHeight, bd.scrollWidth],
        log: logb && vis(logb) ? R(logb) : null, tl: tl ? R(tl) : null, map: R(cv), bad, sy: window.scrollY }; })()`);
    check(m.main && m.main[0] <= m.main[1], `${L}: .main must not overflow, scrollHeight/clientHeight ${JSON.stringify(m.main)}`);
    check(m.doc[0] <= m.doc[1] && m.doc[2] <= m.doc[3] && m.doc[4] <= h && m.doc[5] <= w, `${L}: the document must not scroll, ${JSON.stringify(m.doc)}`);
    check(m.log && m.tl && !(Math.min(m.log.r, m.tl.r) - Math.max(m.log.x, m.tl.x) > 1 && Math.min(m.log.b, m.tl.b) - Math.max(m.log.y, m.tl.y) > 1), `${L}: Event log button ${JSON.stringify(m.log)} must not intersect the timeline ${JSON.stringify(m.tl)}`);
    check(m.log && !(Math.min(m.log.r, m.map.r) - Math.max(m.log.x, m.map.x) > 1 && Math.min(m.log.b, m.map.b) - Math.max(m.log.y, m.map.y) > 1), `${L}: Event log button must not overlap the map`);
    check(m.tl && m.tl.h >= 60, `${L}: the timeline must keep a minimum height of 60 css px, got ${m.tl && m.tl.h}`);
    check(m.bad.length === 0, `${L}: controls must not overlap each other: ${m.bad.join('; ')}`);
    await p.close();
  }

  // ============ 8. second review: committed data, fill, animation, tiles, ticks, summary, zoom, dpr ============
  {
    // 8.1 the committed viewer/data/run.js (what a user opens) shows a map on first load and after reload, file://
    const url = pathToFileURL(join(root, 'viewer', 'index.html')).href;
    const p = await open(url, { w: 1282, h: 722, dpr: 1 });
    let frac = 0; const t0 = Date.now();
    while (Date.now() - t0 < 3000 && frac <= 0.3) { await sleep(150); frac = await p.ev('(window.__t||(' + HELP + ')) && __t.filled()').catch(() => 0); }
    check(frac > 0.3, `committed viewer/data/run.js over file://: map must be drawn on first load, filled ${(+frac).toFixed(2)} (regenerate run.js with the current binary)`);
    const empty = await p.ev(`(() => { const e = document.getElementById('mapempty'); return e && !e.classList.contains('hidden') ? e.textContent : ""; })()`);
    check(!empty, `committed run.js: the map empty state must be hidden, got "${empty}"`);
    check(p.log.errors.length === 0, 'committed run.js: no console errors, got ' + JSON.stringify(p.log.errors));
    await p.S('Page.reload'); await sleep(1200); await p.ev(HELP).catch(() => {});
    check(await p.ev('__t.filled()') > 0.3, 'committed run.js: the map must also be drawn after a reload');
    await p.close();
  }
  for (const [w, h] of VIEWPORTS) {
    const L = `fill ${w}x${h}`;
    const p = await open(BASE, { w, h, dpr: 1 }); await sleep(900);
    const m = await p.ev(`(() => { const R = e => { const r = e.getBoundingClientRect(); return { x: r.left, y: r.top, r: r.right, b: r.bottom, w: r.width, h: r.height }; };
      const v = R(document.getElementById('replay')), c = R(document.getElementById('map')), side = document.querySelector('.side'), sr = R(side);
      const kids = [...side.children].map(R); const used = kids.reduce((a, k) => a + k.h, 0);
      return { v, c, sr, used, last: kids.length ? Math.max(...kids.map(k => k.b)) : 0 }; })()`);
  // ============ 7. theme via the real toggle ============
  {
    const p = await open(BASE, { w: 1540, h: 890, dpr: 1, theme: 'dark' }); await sleep(900);
    const sample = async () => {
      await settle(p, pickFree ? pickFree.idx : 0);
      const g = await p.ev('__t.geom()'), bm = birdMap(g), e = pickFree || EV[0];
      const hc = bm.c(e.hero.x, e.hero.y);
      const th = await p.ev('document.documentElement.getAttribute("data-theme")');
      const accent = await p.ev('__t.tok("--accent")');
      const tile = await p.ev(`__t.px(${hc[0] + bm.ts * 3},${hc[1] + bm.ts * 3})`);
      const hb = await p.ev(`__t.near(${JSON.stringify(accent)}, 60, ${JSON.stringify([hc[0] - bm.ts * 4, hc[1] - bm.ts * 4, hc[0] + bm.ts * 4, hc[1] + bm.ts * 4])})`);
      const hero = await p.ev(`__t.px(${hc[0]},${hc[1]})`);
      const legend = await p.ev(`(() => { const l = document.getElementById('maplegend'); return l ? [...l.querySelectorAll('*')].map(e => getComputedStyle(e).backgroundColor + '|' + getComputedStyle(e).color).join(',') : ''; })()`);
      return { th, accent, tile, hero, hb, legend };
    };
    const a = await sample();
    await p.ev(`document.getElementById('theme-toggle').click()`); await sleep(500);
    const b = await sample();
    check(a.th !== b.th, `#theme-toggle must switch the theme, before ${a.th} after ${b.th}`);
    check(dist(a.tile, b.tile) > 30, `tile colours must change with the theme, ${a.tile} -> ${b.tile}`);
    check(dist(a.hero, b.hero) > 20 || dist(a.accent, b.accent) > 20 || a.hb.n !== b.hb.n, `marker colours must update with the theme (hero ${a.hero} -> ${b.hero})`);
    check(a.legend && a.legend !== b.legend, 'legend colours must update with the theme');
    for (const s of [a, b]) {
      check(s.hb.n > 0, `${s.th}: hero marker (accent) must be drawn`);
      check(contrast(s.accent, s.tile.slice(0, 3)) >= 3, `${s.th}: hero vs tile contrast ${contrast(s.accent, s.tile.slice(0, 3)).toFixed(2)} must be >= 3:1`);
    }
    const stored = await p.ev('localStorage.getItem("jv-theme")');
    check(stored === b.th, `the choice must persist in localStorage jv-theme, got ${stored} want ${b.th}`);
    await p.close();
  }
} catch (e) {
  check(false, 'test harness error: ' + (e && e.stack || e));
}

cleanup();
done();
