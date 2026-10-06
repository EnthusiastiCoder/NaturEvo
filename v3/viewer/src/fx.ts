import type { FightEvent } from './data';
export interface SparkSpec { kind: 'cone' | 'radial' | 'ring'; count: number; color: number; spread: number; dirAngle: number }
