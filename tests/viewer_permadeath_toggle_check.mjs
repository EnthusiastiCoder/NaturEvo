// Permadeath toggle in the Run controls (plans/2026-10-21_hero-map-walking.md, Addendum 4 part 1, viewer).
// Written first, failing. Stub-DOM harness conventions (_viewer_harness.mjs).
//
// Names picked where the plan is silent:
//   static markup inside #run-controls: element id="permadeath-toggle" with role="switch" and a tabindex or button tag.
//   aria-checked is "true" or "false", default "true". Click toggles; if the element is not a <button>, keydown
//   with key " " and "Enter" must toggle it too (a <button> gets this natively, tag check stands in for it).
//   Run and batch requests carry permadeath=true or permadeath=false. State persists in localStorage key
//   "jv-permadeath" ("true"/"false") and is restored on load.
//   /api/config lists permadeath with kind "bool"; the viewer must not render a range slider for it (#knob-permadeath absent).
//   Styling uses tokens: CSS rules for #permadeath-toggle / [role="switch"] use var(--...) and no hex literals,
//   so both themes work through the existing theme tokens.
import { html, load, tick, runner, jsonResp } from './_viewer_harness.mjs';
