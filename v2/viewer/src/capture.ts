// Browser-side capture for record mode: deterministic stepping, PNG sequence, WebM, early/late split-screen.
import { type Fight, parseFight } from './data';
import { RECORD_DEFAULT, Recorder, frameTime, splitCaptions, splitCellSize } from './record';
import { Scene } from './scene';

export interface CaptureSource { canvas: () => HTMLCanvasElement; draw: (t: number) => void; duration: () => number }

export function genFromUrl(u: string): number {
  const m = /gen_(\d+)/.exec(u);
  return m ? Number(m[1]) : 0;
}

/** Two scenes of the same matchup (early vs late generation) composited side by side with captions. */
export async function splitSource(earlyUrl: string, lateUrl: string, w = RECORD_DEFAULT.w, h = RECORD_DEFAULT.h): Promise<CaptureSource> {
  const load = async (u: string): Promise<Fight> => parseFight(await (await fetch(u)).json());
  const [fa, fb] = await Promise.all([load(earlyUrl), load(lateUrl)]);
  const mk = async (f: Fight) => { const s = await Scene.create(document.createElement('div'), { record: true, ...splitCellSize(w, h) }); s.setFight(f); return s; };
  const [sa, sb] = [await mk(fa), await mk(fb)];
  const caps = splitCaptions({ generation: genFromUrl(earlyUrl) }, { generation: genFromUrl(lateUrl) });
  const out = document.createElement('canvas');
  out.width = w; out.height = h;
  const ctx = out.getContext('2d')!;
  const dur = () => Math.max(fa.frames.length / fa.tickHz, fb.frames.length / fb.tickHz);
  const draw = (t: number) => {
    ctx.fillStyle = '#0c120b'; ctx.fillRect(0, 0, w, h);
    ([[sa, fa, 0], [sb, fb, 1]] as const).forEach(([s, f, i]) => {
      s.drawFight(f, Math.min(t, f.frames.length / f.tickHz - 1e-6), { rays: false });
      const cw = w / 2; // each half is rendered at exactly cw x h: no crop, no bands
      ctx.drawImage(s.app.canvas, i * cw, 0, cw, h);
    });
    ctx.fillStyle = '#ffffff22'; ctx.fillRect(w / 2 - 1, 0, 2, h);
    ctx.font = `700 ${Math.round(h / 20)}px system-ui, sans-serif`; ctx.textAlign = 'center'; ctx.textBaseline = 'top';
    caps.forEach((c, i) => {
      const x = w / 4 + (i * w) / 2, y = h / 40;
      ctx.lineWidth = 6; ctx.strokeStyle = '#000c'; ctx.strokeText(c, x, y);
      ctx.fillStyle = '#fff'; ctx.fillText(c, x, y);
    });
  };
  return { canvas: () => out, draw, duration: dur };
}

const toBlob = (c: HTMLCanvasElement) => new Promise<Blob>((res, rej) => c.toBlob((b) => (b ? res(b) : rej(new Error('toBlob failed'))), 'image/png'));

async function makeSink(): Promise<(name: string, d: Uint8Array | Blob) => void> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const pick = (window as any).showDirectoryPicker;
  if (pick) {
    const dir = await pick.call(window, { mode: 'readwrite' });
    return (name, d) => { void dir.getFileHandle(name, { create: true }).then(async (fh: any) => { const w = await fh.createWritable(); await w.write(d); await w.close(); }); };
  }
  return (name, d) => { const a = document.createElement('a'); a.href = URL.createObjectURL(d instanceof Blob ? d : new Blob([d as BlobPart])); a.download = name; a.click(); setTimeout(() => URL.revokeObjectURL(a.href), 1000); };
}

export function installRecorder(src: CaptureSource): void {
  const fps = RECORD_DEFAULT.fps;
  const frameCount = () => Math.floor(src.duration() * fps);
  const step = (n: number) => src.draw(frameTime(n, fps));
  const png = async (n: number) => { step(n); return toBlob(src.canvas()); };
  const api = {
    frameCount, step, png,
    async exportPng(count = frameCount()) {
      const sink = await makeSink();
      await new Recorder(png, sink, fps).exportPng(count);
    },
    /** Frame-stepped WebM: one captured frame per rendered frame, so timing follows the video clock, not real time. */
    async webm(count = frameCount()): Promise<Blob> {
      const stream = src.canvas().captureStream(0);
      const track = stream.getVideoTracks()[0] as MediaStreamTrack & { requestFrame?: () => void };
      const rec = new MediaRecorder(stream, { mimeType: 'video/webm;codecs=vp9', videoBitsPerSecond: 16_000_000 });
      const chunks: Blob[] = [];
      rec.ondataavailable = (e) => chunks.push(e.data);
      const done = new Promise<void>((r) => { rec.onstop = () => r(); });
      rec.start();
      const t0 = performance.now();
      for (let n = 0; n < count; n++) {
        step(n); track.requestFrame?.();
        const wait = t0 + ((n + 1) * 1000) / fps - performance.now(); // pace so the encoder timestamps match 60 fps
        if (wait > 0) await new Promise((r) => setTimeout(r, wait));
      }
      rec.stop(); await done;
      const blob = new Blob(chunks, { type: 'video/webm' });
      const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = 'naturevo.webm'; a.click();
      return blob;
    },
  };
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (window as any).__record = api;
}
