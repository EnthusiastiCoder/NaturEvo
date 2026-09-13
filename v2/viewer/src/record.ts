// Record mode: chrome hidden, fixed-step deterministic frames independent of real time.
export const RECORD_DEFAULT = { w: 1920, h: 1080, fps: 60 };
export const RECORD_SIZES: [number, number][] = [[1280, 720], [1920, 1080], [3840, 2160]];

export function isRecordMode(search: string): boolean {
  return new URLSearchParams(search).get('record') === '1';
}
export function frameTime(n: number, fps = RECORD_DEFAULT.fps): number { return n / fps; }

export function applyRecordMode(root: { dataset: Record<string, string | undefined> }, on: boolean): void {
  if (on) root.dataset.record = '1';
  else delete root.dataset.record;
}

type Render = (t: number, n: number) => Promise<Uint8Array | Blob> | Uint8Array | Blob;
export class Recorder {
  constructor(private render: Render, private sink: (name: string, data: Uint8Array | Blob) => void, private fps = RECORD_DEFAULT.fps) {}
  async exportPng(frameCount: number): Promise<void> {
    for (let n = 0; n < frameCount; n++) {
      const data = await this.render(frameTime(n, this.fps), n);
      this.sink(`frame_${String(n).padStart(6, '0')}.png`, data);
    }
  }
}
export function splitCaptions(a: { generation: number }, b: { generation: number }): [string, string] {
  return [`Early: gen ${a.generation}`, `Late: gen ${b.generation}`];
}

/** Pixi canvas size: record mode is exactly 1920x1080 at resolution 1; otherwise logical 16:9 stage at dpr (max 2). */
export function sceneSize(record: boolean, dpr: number): { w: number; h: number; resolution: number } {
  return record ? { w: RECORD_DEFAULT.w, h: RECORD_DEFAULT.h, resolution: 1 } : { w: 1920, h: 1080, resolution: Math.min(2, dpr || 1) };
}
/** Each split-screen half is rendered at its own native size (no letterboxing). */
export function splitCellSize(w: number, h: number): { w: number; h: number } { return { w: w / 2, h }; }
