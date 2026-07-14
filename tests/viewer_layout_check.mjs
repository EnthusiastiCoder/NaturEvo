// Layout/structure requirements for the viewer app-shell redesign
// (plans/2026-10-20_viewer-redesign.md). Static checks parse the html text; behaviour
// checks use the shared stub-DOM harness. No dependencies.
import { html, load, makeRun, runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_layout_check');
