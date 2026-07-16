// Addendum 3 requirements (plans/2026-10-20_viewer-redesign-v2.md): (1) hero details live in the top
// hero/battle card (no Hero details drawer), drawers in one row; (2) event log is a searchable,
// groupable, windowed modal dialog; (3) seed input wide, non-spinner, validated.
// Derived from the requirements, not from the current code. Real pixel/focus checks: checklist section 16.
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_polish_check');
