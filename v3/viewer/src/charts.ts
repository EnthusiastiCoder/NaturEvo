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
