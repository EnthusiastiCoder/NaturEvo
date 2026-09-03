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
