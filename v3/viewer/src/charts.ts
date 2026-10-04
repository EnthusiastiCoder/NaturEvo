// Inline-SVG charts. The viewBox equals the pixel size so CSS font sizes are real pixels.
import { smooth } from './aggregate';

export type Size = { w: number; h: number };
export type Series = { key: 'hero' | 'monster' | 'draw'; label: string; ys: number[] };
export type Tip = { xs: number[]; s: { l: string; ys: number[] }[]; pct?: boolean };
