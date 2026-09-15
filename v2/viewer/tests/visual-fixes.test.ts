import { describe, it, expect } from 'vitest';
import { displayPlan, fitCamera, hudOffsets, uiScale, separate, VISUAL_MIN_DIST, fightCamera } from '../src/layout';
import { fightStats, statsLine, type Frame } from '../src/interp';
import { lowResSub } from '../src/ground';
