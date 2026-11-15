// Inline-SVG charts. The viewBox equals the pixel size so CSS font sizes are real pixels.
import { smooth } from './aggregate';

export type Size = { w: number; h: number };
export type Key = 'hero' | 'monster' | 'accent';
export type Series = { key: Key; label: string; ys: number[]; lo?: number[]; hi?: number[] };
export type Tip = { xs: number[]; s: { l: string; ys: number[] }[]; pct?: boolean };

const PAD = { l: 34, r: 8, t: 8, b: 20 };
const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
export const pct = (r: number | null) => (r === null ? 'n/a' : `${Math.round(r * 100)}%`);
const num = (v: number) => String(+v.toFixed(2));

export function legendHtml(items: { key: Key; label: string }[]): string {
  return `<div class="legend">${items.map((i) => `<span class="lg"><svg class="sw" viewBox="0 0 22 10" aria-hidden="true"><line class="ln ${i.key}" x1="1" y1="5" x2="21" y2="5"/></svg>${esc(i.label)}</span>`).join('')}</div>`;
}

export function nearestIndex(x: number, n: number, l: number, r: number): number {
  if (n <= 1) return 0;
  return Math.min(n - 1, Math.max(0, Math.round(((x - l) / Math.max(1, r - l)) * (n - 1))));
}
export function tipText(t: Tip, i: number): string {
  const f = (v: number) => (t.pct ? pct(v) : num(v));
  return `gen ${t.xs[i]}\n${t.s.map((s) => `${s.l}: ${f(s.ys[i] ?? 0)}`).join('\n')}`;
}

/** Lines with an optional min-max band per series. */
export function lineChart(series: Series[], xs: number[], yMax: number, size: Size, opts: { pct?: boolean } = {}): string {
  const { w, h } = size, n = xs.length;
  if (n === 0) return '<div class="na">no data</div>';
  const px = (i: number) => PAD.l + (n > 1 ? (i / (n - 1)) * (w - PAD.l - PAD.r) : 0);
  const py = (v: number) => h - PAD.b - (Math.min(yMax, Math.max(0, v)) / yMax) * (h - PAD.t - PAD.b);
  const pts = (ys: number[]) => ys.map((v, i) => `${px(i).toFixed(1)},${py(v).toFixed(1)}`).join(' ');
  const win = n > 40 ? Math.max(3, Math.round(n / 20) | 1) : 1;
  const rev = (ys: number[]) => {
    const p = ys.map((v, i) => `${px(i).toFixed(1)},${py(v).toFixed(1)}`);
    return p.reverse().join(' ');
  };
  const bands = series.filter((s) => s.lo && s.hi).map((s) => `<polygon class="band ${s.key}" points="${pts(smooth(s.hi!, win))} ${rev(smooth(s.lo!, win))}"/>`).join('');
  const lines = series.map((s) => `<polyline class="ln ${s.key}" points="${pts(smooth(s.ys, win))}"/>`).join('');
  const fmt = (v: number) => (opts.pct ? pct(v) : num(v));
  const tip: Tip = { xs, s: series.map((s) => ({ l: s.label, ys: s.ys.map((v) => +v.toFixed(3)) })), pct: opts.pct };
  const grid = [0, 0.5, 1].map((f) => `<line class="grid" x1="${PAD.l}" y1="${py(yMax * f)}" x2="${w - PAD.r}" y2="${py(yMax * f)}"/><text class="tick" x="${PAD.l - 4}" y="${py(yMax * f) + 4}" text-anchor="end">${fmt(yMax * f)}</text>`).join('');
  return `<svg class="chart" viewBox="0 0 ${w} ${h}" role="img" data-l="${PAD.l}" data-r="${w - PAD.r}" data-tip="${esc(JSON.stringify(tip))}">${grid}
<text class="tick" x="${PAD.l}" y="${h - 4}">gen ${xs[0]}</text><text class="tick" x="${w - PAD.r}" y="${h - 4}" text-anchor="end">${xs[n - 1]}</text>${bands}${lines}
<line class="cursor" y1="${PAD.t}" y2="${h - PAD.b}" hidden/></svg>`;
}

export type Bar = { label: string; value: number; tip: string };
/** Bars on a 0-100% axis with a dashed 50% reference line. */
export function barChart(bars: Bar[], size: Size): string {
  const { w, h } = size, base = h - 20, top = 14;
  const bw = (w - PAD.l - PAD.r) / Math.max(1, bars.length), y = (v: number) => base - v * (base - top);
  const g = bars.map((b, i) => {
    const x = PAD.l + i * bw + bw * 0.2, cx = x + bw * 0.3;
    return `<g class="barg" data-tip="${esc(b.tip)}"><rect class="bar" x="${x.toFixed(1)}" y="${y(b.value).toFixed(1)}" width="${(bw * 0.6).toFixed(1)}" height="${(base - y(b.value)).toFixed(1)}"/>
<text class="lbl" x="${cx.toFixed(1)}" y="${(y(b.value) - 4).toFixed(1)}" text-anchor="middle">${pct(b.value)}</text><text class="tick" x="${cx.toFixed(1)}" y="${h - 4}" text-anchor="middle">${esc(b.label)}</text></g>`;
  }).join('');
  return `<svg class="chart" viewBox="0 0 ${w} ${h}" role="img"><line class="axis" x1="${PAD.l}" y1="${base}" x2="${w - PAD.r}" y2="${base}"/>
<line class="grid dash" x1="${PAD.l}" y1="${y(0.5)}" x2="${w - PAD.r}" y2="${y(0.5)}"/><text class="tick" x="${PAD.l - 4}" y="${y(0.5) + 4}" text-anchor="end">50%</text><text class="tick" x="${PAD.l - 4}" y="${top + 4}" text-anchor="end">100%</text>${g}</svg>`;
}
