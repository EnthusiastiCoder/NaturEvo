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
