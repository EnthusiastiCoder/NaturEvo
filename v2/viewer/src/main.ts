import './style.css';
import { type Encounter, type Fight, type Run, demoFight, demoRun, mergeDuels, parseFight, parseRun, resolveFights } from './data';
import { fightStats, statsLine } from './interp';
import { statusStats } from './fx';
import { applyRecordMode, isRecordMode } from './record';
import { Scene } from './scene';
import { installRecorder, splitSource } from './capture';
import { route } from './route';
import { loadIndex } from './runs-index';
import { mountLanding } from './landing';
import { DEFAULT_ACCENT, type Theme, applyTheme, nextTheme, setAccent, themeIconPath } from './theme';

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const WALK_SPEED = 7; // path tiles per second
const ENC_PAUSE = 1.2;

type Seg = { kind: 'walk'; from: number; to: number; t0: number; t1: number } | { kind: 'fight'; enc: number; t0: number; t1: number };

let scene: Scene;
let run: Run = demoRun();
let duelFights: { label: string; fight: Fight }[] = [];
let mode: 'campaign' | 'duel' = 'campaign';
let segs: Seg[] = [];
let total = 1;
let t = 0;
let playing = true;
let speed = 1;
let rays = false;
let current = '';
let duelIdx = 0;

const root = document.documentElement;
let requestedDuel: { label: string; fight: Fight } | null = null;

function banner(msg: string): void {
  const b = $('banner');
  b.textContent = msg; b.hidden = !msg;
}

function buildTimeline(): void {
  segs = [];
  let cursor = 0, tt = 0;
  const idxOf = (e: Encounter) => Math.max(0, run.path.findIndex((p) => p[0] === e.tile[0] && p[1] === e.tile[1]));
  run.encounters.forEach((e, k) => {
    const idx = Math.max(cursor, idxOf(e));
    const d = (idx - cursor) / WALK_SPEED;
    if (d > 0) segs.push({ kind: 'walk', from: cursor, to: idx, t0: tt, t1: tt + d });
    tt += d; cursor = idx;
    const fd = e.fight ? e.fight.frames.length / e.fight.tickHz : ENC_PAUSE;
    segs.push({ kind: 'fight', enc: k, t0: tt, t1: tt + fd });
    tt += fd;
  });
  const end = run.path.length - 1;
  if (end > cursor) { segs.push({ kind: 'walk', from: cursor, to: end, t0: tt, t1: tt + (end - cursor) / WALK_SPEED }); tt += (end - cursor) / WALK_SPEED; }
  total = Math.max(0.1, tt);
}

function populateSelect(): void {
  const sel = $<HTMLSelectElement>('fight-select');
  sel.innerHTML = '';
  if (mode === 'campaign') {
    sel.add(new Option('Whole run', '-1'));
    run.encounters.forEach((e, k) => sel.add(new Option(`${k + 1}. ${e.kind} (zone ${e.zone})`, String(k))));
  } else {
    duelFights.forEach((d, k) => sel.add(new Option(d.label, String(k))));
  }
}

function collectDuels(): void {
  duelFights = mergeDuels(run.encounters.filter((e) => e.fight).map((e, k) => ({ label: `${k + 1}. ${e.kind} vs monster ${e.monsterIndex}`, fight: e.fight! })), requestedDuel);
  if (!duelFights.length) duelFights = [{ label: 'Demo duel', fight: demoFight(1) }];
}

function duration(): number {
  return mode === 'campaign' ? total : duelFights[duelIdx].fight.frames.length / duelFights[duelIdx].fight.tickHz;
}

function strip(f: Fight, tt: number): string {
  const st = fightStats(f.frames, Math.floor(tt * f.tickHz));
  const fx = statusStats(f.frames, Math.floor(tt * f.tickHz));
  const side = (n: string, s: typeof st.hero, x: typeof fx.hero) => {
    const on = Object.entries(x.statuses).filter(([, v]) => v > 0).map(([k, v]) => `${k} ${v}`).join(', ');
    return `${statsLine(n, s)}, ${x.punishes} punish, ${x.parries} parry${on ? ` [${on}]` : ''}`;
  };
  return `${side('Hero', st.hero, fx.hero)}  |  ${side('Monster', st.monster, fx.monster)}`;
}

function frame(): void {
  const dur = duration();
  const scrub = $<HTMLInputElement>('scrub');
  let label = '', a = '', b = '', c = '';
  if (mode === 'campaign') {
    const seg = segs.find((s) => t >= s.t0 && t < s.t1) ?? segs[segs.length - 1];
    if (seg.kind === 'walk') {
      if (current !== 'map') { scene.setMap(run); current = 'map'; }
      const f = Math.min(1, (t - seg.t0) / Math.max(1e-6, seg.t1 - seg.t0));
      scene.drawMap(run, seg.from + (seg.to - seg.from) * f, t, -1);
      label = 'Walking the jungle route';
      a = `Step ${Math.floor(seg.from + (seg.to - seg.from) * f)} / ${run.path.length - 1}`;
    } else {
      const e = run.encounters[seg.enc];
      if (e.fight) {
        const key = `fight${seg.enc}`;
        if (current !== key) { scene.setFight(e.fight); current = key; }
        const s = scene.drawFight(e.fight, t - seg.t0, { rays });
        a = `Hero ${Math.round(s.hero.hp)} HP`; b = `${e.kind} ${Math.round(s.monster.hp)} HP`;
        c = strip(e.fight, t - seg.t0);
      } else {
        if (current !== 'map') { scene.setMap(run); current = 'map'; }
        scene.drawMap(run, run.path.findIndex((p) => p[0] === e.tile[0] && p[1] === e.tile[1]), t, seg.enc);
      }
      label = `Encounter ${seg.enc + 1}: ${e.kind}, zone ${e.zone}${e.result ? ' - ' + e.result : ''}`;
    }
  } else {
    const d = duelFights[duelIdx];
    const key = `duel${duelIdx}`;
    if (current !== key) { scene.setFight(d.fight); current = key; }
    const s = scene.drawFight(d.fight, t, { rays });
    a = `Hero ${Math.round(s.hero.hp)} HP`; b = `Monster ${Math.round(s.monster.hp)} HP`;
    c = strip(d.fight, t);
    label = d.label;
  }
  $('hud-a').textContent = label;
  $('hud-b').textContent = [a, b].filter(Boolean).join('  |  ');
  $('hud-c').textContent = c; $('hud-c').hidden = !c;
  scrub.value = String(Math.round((t / dur) * 1000));
  $('clock').textContent = `${t.toFixed(1)} / ${dur.toFixed(1)} s`;
}

let last = performance.now();
function loop(now: number): void {
  const dt = Math.min(0.1, (now - last) / 1000);
  last = now;
  const dur = duration();
  if (playing) {
    t += dt * speed;
    if (t >= dur) { t = dur - 1e-6; playing = false; syncPlay(); }
  }
  frame();
  requestAnimationFrame(loop);
}

function syncPlay(): void {
  const b = $('play');
  b.innerHTML = playing
    ? '<svg viewBox="0 0 24 24"><path d="M6 4h4v16H6zM14 4h4v16h-4z"/></svg>'
    : '<svg viewBox="0 0 24 24"><path d="m6 4 14 8-14 8z"/></svg>';
}

function setMode(m: 'campaign' | 'duel'): void {
  mode = m; t = 0; current = ''; duelIdx = 0;
  $('mode-walk').setAttribute('aria-pressed', String(m === 'campaign'));
  $('mode-duel').setAttribute('aria-pressed', String(m === 'duel'));
  populateSelect();
}

async function loadFiles(files: FileList): Promise<void> {
  for (const f of Array.from(files)) {
    const j = JSON.parse(await f.text());
    if (j.map && j.path) { run = parseRun(j); const errs = await resolveFights(run, location.href); banner(errs.length ? `Could not load fights: ${errs.join('; ')}` : ''); buildTimeline(); collectDuels(); setMode('campaign'); }
    else if (j.frames) { duelFights = [{ label: f.name, fight: parseFight(j) }]; setMode('duel'); }
  }
}

async function tryLoadDefault(): Promise<void> {
  const q = new URLSearchParams(location.search);
  const req = q.get('run');
  // A bare run name (as listed on the landing) maps to its campaign file; paths and URLs are used as given.
  const runPath = req && !/\.json$|^\/|^https?:/i.test(req) ? `/runs/${req}/campaign_on.json` : req;
  const runUrl = new URL(runPath ?? '/data/run.json', location.href).href;
  const problems: string[] = [];
  let loaded = false;
  try {
    const r = await fetch(runUrl);
    if (r.ok && (r.headers.get('content-type') ?? '').includes('json')) {
      run = parseRun(await r.json()); loaded = true;
      const errs = await resolveFights(run, runUrl);
      if (errs.length) problems.push(`Could not load fights: ${errs.join('; ')}`);
    } else problems.push(`No run found at ${q.get('run') ?? '/data/run.json'} (HTTP ${r.status}); showing the built-in demo.`);
  } catch (err) { problems.push(`Could not read run ${runUrl}: ${err instanceof Error ? err.message : String(err)}; showing the built-in demo.`); }
  const fu = q.get('fight');
  if (fu) {
    try {
      const r = await fetch(new URL(fu, location.href).href);
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      requestedDuel = { label: `Requested: ${fu}`, fight: parseFight(await r.json()) };
    } catch (err) { problems.push(`Could not load ?fight=${fu}: ${err instanceof Error ? err.message : String(err)}`); }
  }
  if (!loaded && !q.get('run') && requestedDuel) problems.shift(); // an explicit fight makes the missing default run expected
  banner(problems.join(' | '));
}

function fitNow(): void {
  const st = $('view');
  if (scene.fit(st.clientWidth, st.clientHeight)) current = ''; // ground texture density follows the display size
}

let demo = false;

async function main(): Promise<void> {
  let theme: Theme = matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  const stored = localStorage.getItem('nv-theme');
  if (stored === 'light' || stored === 'dark') theme = stored;
  applyTheme(root, theme);
  const acc = localStorage.getItem('nv-accent') ?? DEFAULT_ACCENT;
  setAccent(root, acc); $<HTMLInputElement>('accent').value = acc;
  applyRecordMode(root, isRecordMode(location.search));
  const syncThemeIcon = () => { $('theme-toggle').innerHTML = `<svg viewBox="0 0 24 24">${themeIconPath(theme)}</svg>`; };
  syncThemeIcon();
  $('theme-toggle').onclick = () => { theme = nextTheme(theme); applyTheme(root, theme); localStorage.setItem('nv-theme', theme); syncThemeIcon(); };
  $<HTMLInputElement>('accent').oninput = (e) => { const v = (e.target as HTMLInputElement).value; setAccent(root, v); localStorage.setItem('nv-accent', v); };

  scene = await Scene.create($('view'), { record: isRecordMode(location.search) });
  const startViewer = async (): Promise<void> => {
    delete root.dataset.view;
  if (!demo) await tryLoadDefault();
  buildTimeline(); collectDuels();
  if (requestedDuel) setMode('duel'); else populateSelect();
  fitNow();
  new ResizeObserver(fitNow).observe($('stage'));

  $('play').onclick = () => { if (t >= duration() - 1e-3) t = 0; playing = !playing; syncPlay(); };
  $('restart').onclick = () => { t = 0; playing = true; syncPlay(); };
  $<HTMLInputElement>('scrub').oninput = (e) => { t = (Number((e.target as HTMLInputElement).value) / 1000) * duration(); };
  $<HTMLSelectElement>('speed').onchange = (e) => { speed = Number((e.target as HTMLSelectElement).value); };
  $('mode-walk').onclick = () => setMode('campaign');
  $('mode-duel').onclick = () => setMode('duel');
  $<HTMLSelectElement>('fight-select').onchange = (e) => {
    const v = Number((e.target as HTMLSelectElement).value);
    if (mode === 'campaign') { const s = segs.find((x) => x.kind === 'fight' && x.enc === v); t = v < 0 ? 0 : s ? s.t0 : 0; }
    else { duelIdx = v; t = 0; current = ''; }
    playing = true; syncPlay();
  };
  $('rays').onclick = () => toggleRays();
  $('load').onclick = () => $<HTMLInputElement>('file').click();
  $<HTMLInputElement>('file').onchange = (e) => { const f = (e.target as HTMLInputElement).files; if (f) void loadFiles(f); };
  $('help').onclick = () => { const i = $('info'); i.hidden = !i.hidden; };
  addEventListener('keydown', (e) => {
    if ((e.target as HTMLElement).tagName === 'INPUT' && (e.target as HTMLInputElement).type !== 'range') return;
    if (e.key === ' ' && (e.target as HTMLElement).tagName !== 'BUTTON') { e.preventDefault(); $('play').click(); }
    else if (e.key === 'ArrowRight') t = Math.min(duration(), t + 0.5);
    else if (e.key === 'ArrowLeft') t = Math.max(0, t - 0.5);
    else if (e.key === 't') $('theme-toggle').click();
    else if (e.key === 'v') toggleRays();
    else if (e.key === '?') $('help').click();
    else if (e.key === 'Escape') $('info').hidden = true;
  });
  syncPlay();
  if (isRecordMode(location.search)) {
    const q = new URLSearchParams(location.search);
    const [e, l] = [q.get('early'), q.get('late')];
    if (e && l) {
      const src = await splitSource(e, l);
      scene.app.canvas.style.display = 'none';
      const c = src.canvas(); c.style.cssText = 'width:100%;height:100%;object-fit:contain';
      $('view').appendChild(c);
      installRecorder(src); src.draw(0);
      return;
    }
    playing = false;
    frame(); // first frame is visible without calling __record.step
    installRecorder({ canvas: () => scene.app.canvas as HTMLCanvasElement, duration, draw: (tt) => { t = Math.min(tt, duration() - 1e-6); frame(); } });
    return;
  }
  requestAnimationFrame(loop);
  };
  if (route(location.search) === 'landing') {
    root.dataset.view = 'landing';
    const index = await loadIndex();
    await mountLanding($('landing'), index, () => { demo = true; void startViewer(); });
    return;
  }
  await startViewer();
}

function toggleRays(): void { rays = !rays; $('rays').setAttribute('aria-pressed', String(rays)); }

void main();
