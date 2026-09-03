// Assumed module: src/record.ts
//  isRecordMode(search: string): boolean
//  RECORD_DEFAULT = {w:1920,h:1080,fps:60}; RECORD_SIZES = [[1280,720],[1920,1080],[3840,2160]]
//  frameTime(n: number, fps?: number): number            (seconds, = n/fps, no real-time clock)
//  class Recorder { constructor(renderFrame:(t:number,n:number)=>Promise<Uint8Array|Blob>|..., sink:(name:string,data:any)=>void) ;
//                   async exportPng(frameCount:number): Promise<void> }  sink called once per frame, names frame_000000.png
//  splitCaptions(a:{generation:number}, b:{generation:number}): [string,string]
//  chromeHidden(doc): see app test below uses applyRecordMode(root, on) toggling data-record on root.
import { describe, it, expect } from 'vitest';
import { isRecordMode, RECORD_DEFAULT, RECORD_SIZES, frameTime, Recorder, splitCaptions, applyRecordMode } from '../src/record';

describe('record mode', () => {
  it('?record=1 enables, other queries do not', () => {
    expect(isRecordMode('?record=1')).toBe(true);
    expect(isRecordMode('?x=2&record=1')).toBe(true);
    expect(isRecordMode('')).toBe(false);
    expect(isRecordMode('?record=0')).toBe(false);
  });
  it('default resolution 1920x1080 at 60 fps, options 720p and 4K', () => {
    expect(RECORD_DEFAULT).toEqual({ w: 1920, h: 1080, fps: 60 });
    expect(RECORD_SIZES).toContainEqual([1280, 720]);
    expect(RECORD_SIZES).toContainEqual([3840, 2160]);
  });
  it('chrome is hidden in record mode and restored after', () => {
    const root = { dataset: {} as Record<string, string>, classList: { toggle() {} } } as any;
    applyRecordMode(root, true);
    expect(root.dataset.record).toBe('1');
    applyRecordMode(root, false);
    expect(root.dataset.record).toBeUndefined();
  });
  it('frame N time is exactly N/60, independent of real time', () => {
    expect(frameTime(0)).toBe(0);
    expect(frameTime(60)).toBe(1);
    expect(frameTime(90)).toBeCloseTo(1.5, 12);
    const a = frameTime(123);
    // burn real time, result unchanged
    const t0 = Date.now(); while (Date.now() - t0 < 5) {}
    expect(frameTime(123)).toBe(a);
  });
  it('rendering frame N twice gives identical bytes (deterministic callback contract)', async () => {
    const calls: number[] = [];
    const render = async (t: number, n: number) => { calls.push(n); return new Uint8Array([n & 255, Math.round(t * 1000) & 255]); };
    const out: Record<string, any> = {};
    const r = new Recorder(render, (name, data) => { out[name] = data; });
    await r.exportPng(5);
    const first = { ...out };
