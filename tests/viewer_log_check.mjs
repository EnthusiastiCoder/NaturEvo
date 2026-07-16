// CHANGED DELIBERATELY (plans/2026-10-20_viewer-redesign-v2.md, Addendum 3): the event log is now a modal
// (#log-open -> #log-modal) that lists ALL events (search/grouping need the full set), windowed to <=300
// rows, with the current row highlighted (.cur) and scrolled into view. The old behaviour "log lists only
// 0..cur" (and its "append on step" window checks) is retired. Kept: #log is position:relative and the
// scroller, __jv.logIndices() returns numbers (the rendered indices), the .cur highlight + scroll-into-view,
// stepping forward does not rebuild the list. Grouping is set to "none" so rows are in event order.
import { html, load, makeRun, runner } from './_viewer_harness.mjs';
