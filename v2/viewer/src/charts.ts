// Inline-SVG charts. The viewBox equals the pixel size so CSS font sizes are real pixels.
import { smooth } from './aggregate';

export type Size = { w: number; h: number };
export type Series = { key: 'hero' | 'monster' | 'draw'; label: string; ys: number[] };
export type Tip = { xs: number[]; s: { l: string; ys: number[] }[]; pct?: boolean };

const PAD = { l: 34, r: 8, t: 8, b: 20 };
const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
export const pct = (r: number | null) => (r === null ? 'n/a' : `${Math.round(r * 100)}%`);
const num = (v: number) => String(+v.toFixed(2));
