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
