// Obstacles must be clearly visible on the 2D map (plans/2026-10-21_hero-map-walking.md, Addendum 4 part 3).
// Written first, failing. Real pixels: like viewer_map_review_check.mjs this drives a real headless Edge over the
// DevTools protocol (node's built-in WebSocket) against the real server, with the runner() conventions of
// _viewer_harness.mjs (check()/done(), "FAIL: ..." lines, exit code 1, no dependencies).
//
// Names/geometry used (the implementation must match or this test is updated):
//   #map canvas, #mapzoom (Zoom button), #theme-toggle (real toggle, clicked), #maplegend (children = legend entries;
//   swatch = a child element with class sw-wall sw-rock sw-fence sw-tree sw-water, either a <canvas> (centre pixel)
//   or an element with a CSS background-color), tokens --tile-floor --tile-grass --tile-rock --tile-fence --tile-tree
//   --tile-water --tile-wall in both themes. Tile chars from /api/map: '#' wall, 'R' rock, 'F' fence, 'T' tree,
//   'W' water, '.' floor, ',' grass, 'C' camp; zones are digit chars.
//   Bird's-eye geometry (device px): ts = floor(min(W,H)/mapW), ox = floor((W-ts*mapW)/2), oy likewise.
//   Zoom: 16x16 tile window, ts = W/16, top-left tile = clamp(hero - 8, 0, mapW - 16).
//   Contrast is WCAG relative-luminance contrast >= 3:1; kinds differ pairwise by RGB distance >= 25.
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, cpSync, readFileSync, rmSync } from 'node:fs';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_obstacles_visible_check');
const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const sleep = ms => new Promise(r => setTimeout(r, ms));
