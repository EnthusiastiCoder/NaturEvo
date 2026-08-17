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

const { check, done } = runner('viewer_obstacle_check');
const N = 40;
const KINDS = [['#', 'Wall', '--tile-wall'], ['R', 'Rock', '--tile-rock'], ['F', 'Fence', '--tile-fence'], ['T', 'Tree', '--tile-tree'], ['W', 'Water', '--tile-water']];
const row20 = '.'.repeat(4) + '#.R.F.T.W' + '.'.repeat(N - 13);
const tiles = Array.from({ length: N }, (_, y) => y === 20 ? row20 : '.'.repeat(N)).map((r, y) => y === 0 ? 'C' + r.slice(1) : r);
const zones = Array.from({ length: N }, () => '1'.repeat(N));
const legend = [['#', 'Wall', true], ['R', 'Rock', true], ['F', 'Fence', true], ['T', 'Tree', true], ['W', 'Water', true], ['.', 'Floor', false], [',', 'Grass', false], ['C', 'Camp', false]].map(([ch, name, blocking]) => ({ ch, name, blocking }));
const MAP = { w: N, h: N, tiles, zones, legend, anchors: Array.from({ length: 9 }, (_, i) => [i * 4, 5]) };
const hero = (x, y) => ({ level: 5, hp: 20, max_hp: 20, xp: 0, gold: 0, potions: 0, zone: 1, weapon: 0, armor: 0, x, y });
const RUN = { seed: 1, won: false, deaths: 0, final_level: 5, w: N, h: N, steps: 9, events: [0, 1, 2].map(i => ({ idx: i, kind: 'travel', text: 'e' + i, hero: hero(10, 10), mon: null, npcs: [] })) };
const norm = c => String(c || '').trim().toLowerCase().replace(/\s+/g, '');
