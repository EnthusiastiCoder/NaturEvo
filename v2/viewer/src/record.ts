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
