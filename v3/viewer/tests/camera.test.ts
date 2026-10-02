// Assumed module: src/camera.ts exports cameraFor(frame, w, h) -> {scale, tx, ty}
// World coordinates are cm, y up; screen = (x*scale + tx, ty - y*scale).
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight } from '../src/data';
import { cameraFor } from '../src/camera';
