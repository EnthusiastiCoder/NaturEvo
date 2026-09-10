// Browser-side capture for record mode: deterministic stepping, PNG sequence, WebM, early/late split-screen.
import { type Fight, parseFight } from './data';
import { RECORD_DEFAULT, Recorder, frameTime, splitCaptions } from './record';
import { Scene } from './scene';

export interface CaptureSource { canvas: () => HTMLCanvasElement; draw: (t: number) => void; duration: () => number }

export function genFromUrl(u: string): number {
  const m = /gen_(\d+)/.exec(u);
  return m ? Number(m[1]) : 0;
}
