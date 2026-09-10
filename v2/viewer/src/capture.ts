// Browser-side capture for record mode: deterministic stepping, PNG sequence, WebM, early/late split-screen.
import { type Fight, parseFight } from './data';
import { RECORD_DEFAULT, Recorder, frameTime, splitCaptions } from './record';
import { Scene } from './scene';
