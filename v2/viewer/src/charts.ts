// Inline-SVG charts. The viewBox equals the pixel size so CSS font sizes are real pixels.
import { smooth } from './aggregate';

export type Size = { w: number; h: number };
export type Series = { key: 'hero' | 'monster' | 'draw'; label: string; ys: number[] };
export type Tip = { xs: number[]; s: { l: string; ys: number[] }[]; pct?: boolean };

const PAD = { l: 34, r: 8, t: 8, b: 20 };
const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
export const pct = (r: number | null) => (r === null ? 'n/a' : `${Math.round(r * 100)}%`);
const num = (v: number) => String(+v.toFixed(2));

export function legendHtml(items: { key: string; label: string }[]): string {
  return `<div class="legend">${items.map((i) => `<span class="lg ${i.key}"><svg class="sw" viewBox="0 0 22 10" aria-hidden="true"><line class="ln ${i.key}" x1="1" y1="5" x2="21" y2="5"/></svg>${esc(i.label)}</span>`).join('')}</div>`;
}

export function nearestIndex(x: number, n: number, l: number, r: number): number {
  if (n <= 1) return 0;
  return Math.min(n - 1, Math.max(0, Math.round(((x - l) / Math.max(1, r - l)) * (n - 1))));
}

export function tipText(t: Tip, i: number): string {
  const f = (v: number) => (t.pct ? pct(v) : num(v));
  return `gen ${t.xs[i]}\n${t.s.map((s) => `${s.l}: ${f(s.ys[i] ?? 0)}`).join('\n')}`;
}

export function lineChart(series: Series[], xs: number[], yMax: number, size: Size, opts: { pct?: boolean } = {}): string {
  const { w, h } = size, n = xs.length;
  const px = (i: number) => PAD.l + (n > 1 ? (i / (n - 1)) * (w - PAD.l - PAD.r) : 0);
  const py = (v: number) => h - PAD.b - (Math.min(yMax, Math.max(0, v)) / yMax) * (h - PAD.t - PAD.b);
  const pts = (ys: number[]) => ys.map((v, i) => `${px(i).toFixed(1)},${py(v).toFixed(1)}`).join(' ');
  const win = Math.max(3, Math.round(n / 20) | 1);
  const lines = series.map((s) => `<polyline class="ln raw ${s.key}" points="${pts(s.ys)}"/>`).join('')
    + series.map((s) => `<polyline class="ln ${s.key}" points="${pts(smooth(s.ys, win))}"/>`).join('');
  const fmt = (v: number) => (opts.pct ? pct(v) : num(v));
  const tip: Tip = { xs, s: series.map((s) => ({ l: s.label, ys: s.ys.map((v) => +v.toFixed(3)) })), pct: opts.pct };
  const grid = [0, 0.5, 1].map((f) => `<line class="grid" x1="${PAD.l}" y1="${py(yMax * f)}" x2="${w - PAD.r}" y2="${py(yMax * f)}"/><text class="tick" x="${PAD.l - 4}" y="${py(yMax * f) + 4}" text-anchor="end">${fmt(yMax * f)}</text>`).join('');
  return `<svg class="chart" viewBox="0 0 ${w} ${h}" role="img" data-l="${PAD.l}" data-r="${w - PAD.r}" data-tip="${esc(JSON.stringify(tip))}">${grid}
<text class="tick" x="${PAD.l}" y="${h - 4}">gen ${xs[0]}</text><text class="tick" x="${w - PAD.r}" y="${h - 4}" text-anchor="end">${xs[n - 1]}</text>${lines}
<line class="cursor" y1="${PAD.t}" y2="${h - PAD.b}" hidden/></svg>`;
}

export type Bar = { label: string; k: number; n: number; rate: number | null };
export function barChart(bars: Bar[], size: Size): string {
  const { w, h } = size, b2 = 30, top = 22;
  const bw = (w - PAD.l - PAD.r) / Math.max(1, bars.length), base = h - b2;
  const g = bars.map((b, i) => {
    const bh = b.rate === null ? 0 : b.rate * (base - top), x = PAD.l + i * bw + bw * 0.18, cx = x + bw * 0.32;
    return `<g class="barg" data-tip="${esc(`${b.label}\n${b.k} of ${b.n} won (${pct(b.rate)})`)}"><rect class="bar" x="${x.toFixed(1)}" y="${(base - bh).toFixed(1)}" width="${(bw * 0.64).toFixed(1)}" height="${bh.toFixed(1)}"/>
<text class="lbl" x="${cx.toFixed(1)}" y="${(base - bh - 4).toFixed(1)}" text-anchor="middle">${pct(b.rate)}</text>
<text class="tick" x="${cx.toFixed(1)}" y="${base + 13}" text-anchor="middle">${esc(b.label)}</text><text class="tick" x="${cx.toFixed(1)}" y="${base + 26}" text-anchor="middle">${b.k}/${b.n}</text></g>`;
  }).join('');
  return `<svg class="chart" viewBox="0 0 ${w} ${h}" role="img"><line class="axis" x1="${PAD.l}" y1="${base}" x2="${w - PAD.r}" y2="${base}"/>
<text class="tick" x="${PAD.l - 4}" y="${base + 4}" text-anchor="end">0</text><text class="tick" x="${PAD.l - 4}" y="${top + 4}" text-anchor="end">100%</text>${g}</svg>`;
}
