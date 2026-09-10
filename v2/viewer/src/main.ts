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
