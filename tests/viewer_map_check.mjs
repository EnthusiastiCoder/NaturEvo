// Map canvas requirements (plans/2026-10-21_hero-map-walking.md, workstream 3).
// Written from the plan before the implementation, using the stub-DOM harness (_viewer_harness.mjs).
//
// Names picked where the plan is silent (the implementation must use these, or this test is updated):
//   #map canvas, #mapzoom button (Zoom toggle), #maptip tooltip element (class "hidden" when not shown),
//   #mapempty element (empty state, class "hidden" when a map is shown),
//   localStorage "jv-mapzoom" = "1" (zoomed) / "0" (bird's-eye),
//   GET /api/map?seed=<n>&map_size=<n> is fetched over http: (JSON shape of the plan, section 9),
//   markers are drawn with fillRect/strokeRect/arc using fillStyle/strokeStyle = tok('--accent') (hero),
//   tok('--warn') (elite npc, id starting "e"), tok('--bad') (npc id "boss").
// Pixel positions are derived from the drawn markers themselves (the harness has no real pixels).
import { html, load, tick, runner, jsonResp } from './_viewer_harness.mjs';
