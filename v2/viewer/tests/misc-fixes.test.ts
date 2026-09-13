import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { sceneSize, splitCellSize } from '../src/record';
import { themeIconPath } from '../src/theme';
import { groundPixels, lowResSub } from '../src/ground';
import { STAGE_W, STAGE_H } from '../src/layout';
