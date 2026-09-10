// Browser-side capture for record mode: deterministic stepping, PNG sequence, WebM, early/late split-screen.
import { type Fight, parseFight } from './data';
import { RECORD_DEFAULT, Recorder, frameTime, splitCaptions } from './record';
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
  const mk = async (f: Fight) => { const s = await Scene.create(document.createElement('div')); s.setFight(f); return s; };
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
      const cw = w / 2, ch = cw * (s.app.canvas.height / s.app.canvas.width);
      ctx.drawImage(s.app.canvas, i * cw, (h - ch) / 2, cw, ch);
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
