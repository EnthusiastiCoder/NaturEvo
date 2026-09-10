import './style.css';
import { type Encounter, type Fight, type Run, demoFight, demoRun, parseFight, parseRun } from './data';
import { RECORD_DEFAULT, applyRecordMode, isRecordMode } from './record';
import { Scene } from './scene';
import { DEFAULT_ACCENT, type Theme, applyTheme, nextTheme, setAccent } from './theme';

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
  duelFights = run.encounters.filter((e) => e.fight).map((e, k) => ({ label: `${k + 1}. ${e.kind} vs monster ${e.monsterIndex}`, fight: e.fight! }));
  if (!duelFights.length) duelFights = [{ label: 'Demo duel', fight: demoFight(1) }];
}

function duration(): number {
  return mode === 'campaign' ? total : duelFights[duelIdx].fight.frames.length / duelFights[duelIdx].fight.tickHz;
}

function frame(): void {
  const dur = duration();
  const scrub = $<HTMLInputElement>('scrub');
  let label = '', a = '', b = '';
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
    label = d.label;
  }
  $('hud-a').textContent = label;
  $('hud-b').textContent = [a, b].filter(Boolean).join('  |  ');
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
    if (j.map && j.path) { run = parseRun(j); await resolveFights(); buildTimeline(); collectDuels(); setMode('campaign'); }
    else if (j.frames) { duelFights = [{ label: f.name, fight: parseFight(j) }]; setMode('duel'); }
  }
}
