// Style-system / icon / tooltip / dashboard requirements for the viewer redesign v2
// (plans/2026-10-20_viewer-redesign-v2.md, "Test cases"). Derived from requirements, not
// from the current layout. Static checks parse viewer/index.html; behaviour checks use the
// shared stub-DOM harness. No dependencies.
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';
