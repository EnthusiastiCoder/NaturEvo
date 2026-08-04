// Map review fixes (plans/2026-10-21_hero-map-walking.md, "Addendum 2"). Written first, failing.
//
// The stub-DOM harness (_viewer_harness.mjs) has no layout engine and no canvas pixels, and these
// requirements are about real pixels and real boxes. So this file keeps the harness conventions
// (runner(): check()/done(), "FAIL: ..." lines, exit code 1 on failure, no dependencies) but drives a
// real headless Edge over the DevTools protocol (node's built-in WebSocket) against the real server:
//   - rebuilds sim (cargo build --release) and generates a seed-1 run into a temp copy of viewer/
//   - serves that copy with `jungle-sim serve --root <copy>` (so a missing route is a real 404)
//   - each scenario runs in an isolated browser context (fresh localStorage), with an exact
//     viewport and device pixel ratio.
//
// Names the implementation must use where the plan is silent (or this test is updated):
//   #map canvas inside #replay (existing), #mapzoom (existing Zoom button), #theme-toggle (existing),
//   #log-open (existing Event log button), #timeline (existing canvas),
//   #maplegend: legend container with exactly 6 entries (direct children), whose text names
//     hero, elite, boss, grass, wall, camp (any case),
//   window.__mapAnimStep(t): renders the attack frame at t in [0,1] with no timers; afterwards
//     window.__mapAnim.pos (hero) and .mpos (monster) are the drawn positions in tile units,
//   window.__mapAnim.active is true synchronously after __jv.goto(i) onto an attack/skill/potion/
//     hit_taken event with a mon (speed 1x), kind is the same string for every attack.
// Pixel geometry (device px, canvas W x H, square): bird's-eye tile ts = floor(min(W,H)/mapW),
//   ox = floor((W-ts*mapW)/2); zoom: 16x16 tile window, ts = W/16, hero tile centred.
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, cpSync, readFileSync, rmSync } from 'node:fs';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { pathToFileURL } from 'node:url';
import { runner } from './_viewer_harness.mjs';
