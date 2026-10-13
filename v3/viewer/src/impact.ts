// Pure impact-effect maths for the fight view. Everything is a function of tick age, so scrubbing and pause stay deterministic.
import type { Seg } from './data';

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
