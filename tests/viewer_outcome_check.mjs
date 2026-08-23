// Outcome badge, dashboard death stats and replay stop (plans/2026-10-21_hero-map-walking.md, Addendum 4 part 1, viewer).
// Written first, failing. Stub-DOM harness conventions (_viewer_harness.mjs).
//
// Names picked where the plan is silent:
//   run head field `outcome` is "won" | "died" | "capped"; badge text "Victory" | "Fallen" | "Capped".
//   The existing #outcome badge in the summary strip carries the text when the replay is at the final event
//   (before that it must not show an outcome word). Old runs without `outcome` fall back to the `won` flag:
//   won -> Victory, not won and capped -> Capped, otherwise Fallen. No exception for missing fields.
//   Dashboard (#dstats) shows death stats from STATS.died / died_pct: text matches /fallen|died|death/i and
//   contains the died_pct with one decimal and a percent sign (e.g. "62.5%").
//   Replay stop: the final event of a died run is kind "end" after kind "death"; play stops there (play button
//   aria-label back to "Play", progress stays at the last index when the interval fires again).
import { load, tick, runner } from './_viewer_harness.mjs';
