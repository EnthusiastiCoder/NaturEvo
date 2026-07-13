// Addendum 2 requirements (plans/2026-10-20_viewer-redesign-v2.md): no clipped content,
// consistent right-aligned selects, tuning knobs as range + synced number input in a
// non-scrolling multi-column grid. Derived from requirements; static CSS/markup checks plus
// behaviour checks with the shared stub-DOM harness. Real pixel checks: tests/viewer_layout_checklist.md section 15.
// UPDATED for Addendum 3b: #speed is no longer a <select> (log range slider + number input). The "Speed select
// exists" and "#speed must use class select" checks were replaced; any remaining selects must still use .select.
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';
