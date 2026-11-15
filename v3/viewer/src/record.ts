// Record mode: chrome hidden, fixed-step deterministic frames independent of real time.
export const RECORD_DEFAULT = { w: 1920, h: 1080, fps: 60 };
export const FILM_GENS = [0, 10, 30, 80, 200];

export function isRecordMode(search: string): boolean { return new URLSearchParams(search).get('record') === '1'; }
export function frameTime(n: number, fps = RECORD_DEFAULT.fps): number { return n / fps; }
export function applyRecordMode(root: { dataset: Record<string, string | undefined> }, on: boolean): void {
  if (on) root.dataset.record = '1'; else delete root.dataset.record;
}

/** The learning film: gens 0, 10, 30, 80, 200 limited to the run length; gen 0 is always kept. */
export function planFilmSequence(maxGen: number): { gen: number; caption: string }[] {
  return FILM_GENS.filter((g) => g === 0 || g <= maxGen).map((gen) => ({ gen, caption: gen === 0 ? 'Generation 0: untrained' : `Generation ${gen}` }));
}
