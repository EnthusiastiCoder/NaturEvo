// Assumed module: src/damage-style.ts exports partDamageStyle(hp: number /*0..100 percent*/, destroyed?: boolean)
//   -> {scratches: boolean, desaturate: number /*0..1*/, cracks: boolean, limp: boolean, detached: boolean}
import { describe, it, expect } from 'vitest';
import { partDamageStyle } from '../src/damage-style';
