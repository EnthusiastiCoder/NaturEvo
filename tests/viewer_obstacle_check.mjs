// Obstacle types in the map viewer (plans/2026-10-21_hero-map-walking.md, Addendum 3 part A, viewer).
// Written first; fails until the feature exists. Stub-DOM harness conventions of viewer_map_check.mjs.
//
// Names picked where the plan is silent:
//   tile chars '#' Wall, 'R' Rock, 'F' Fence, 'T' Tree, 'W' Water (plus '.' ',' 'C'), /api/map adds `legend`.
//   tokens --tile-wall --tile-rock --tile-fence --tile-tree --tile-water in both themes (+ existing --tile-floor).
//   each blocking tile is painted with its token fill (a fillRect covering the tile) and then at least one more
//   drawing op inside that tile (the pattern: rock blob, fence rails/posts, canopy, wave stripes, brick lines).
//   legend swatches in #maplegend use classes sw-wall sw-rock sw-fence sw-tree sw-water sw-floor sw-grass sw-camp
//   (8 tile entries, in markup or built from MAP.legend; swatch classes must exist in the final DOM/markup).
//   tooltip (#maptip) text contains the kind name: Wall, Rock, Fence, Tree, Water.
import { html, load, tick, runner, jsonResp } from './_viewer_harness.mjs';
