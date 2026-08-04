## Goal
Replace zone teleporting with a deterministic tile map. The hero walks tile by tile with A*, meets enemies Pokemon-style (tall-grass random encounters plus visible roaming elites and the boss), and both hero and NPCs have (x,y) in the event log. Monsters get a small weighted move list. The viewer draws the map. The existing event log, windowed API, post-game stages (stage_hist) and 2^16 event cap keep working.

## Standards and constraints
User request, verbatim: "we need the main character to move across a map, not just between zones. gotta have coordinates for both hero and the npcs. this stuff, just copy from pokemon spawning etc. and does the hero have specific moveset?"
Answers:
- Map = tile grid with zone biomes (e.g. 64x64, each tile belongs to a zone/biome, hero walks tile by tile, deterministic from seed).
- Enemies = BOTH: Pokemon tall-grass random encounters (per-step chance on encounter tiles of a zone, species by weighted table) PLUS visible roaming elite NPCs and the boss at fixed/roaming positions with coordinates that move.
- Hero AI = goal-driven A* pathfinding to its target zone, wanders inside to grind, walks back to camp to rest.
- Moveset = keep the hero's existing level-gated auto skills (Power Strike, Brace, Stun, Heal) and add a small weighted move list per monster species (monsters currently only have a passive Behavior enum). The answer to "does the hero have a specific moveset" is yes, the existing four skills (documented in the README, unchanged).
Project rules (CLAUDE.md):
- Never commit to main. Work on feat/jungle-sim or a child branch.
- Tests are written before implementation, from the requirements.
- Commit one file at a time with `git add` and `git commit` run as separate commands. Check `git diff` first. No Co-Authored-By lines in commits.
- Zero dependencies, deterministic seeded RNG (sim/src/rng.rs), no wall-clock input.
Viewer rules (viewer/STYLE.md):
- Single file viewer/index.html, no build, no CDN.
- No hex/rgb literals outside the token blocks, no px font sizes, no inline style= in markup.
- New colours become tokens in BOTH themes.
- The page never scrolls.
- Canvas is crisp (DPR aware, no smoothing). Use watchDpr/fitCanvas and the existing tok() helper.
Backwards compatibility:
- The event kinds, the windowed API (`/api/run?from&count`), stage_hist and event_cap semantics stay.
- Existing Rust tests in sim/tests and tests/*.mjs must pass unless a test pins exact numbers that the new RNG stream legitimately changes. Update only those, in their own commit, with a reason.
- Lean and full mode must give identical summaries (existing invariant): the RNG stream must not depend on `lean`.

## Design defaults (small decisions, resolved)
1. Map generation:
   - Module sim/src/map.rs. `Map::generate(seed, size)` uses its own Rng seeded from seed ^ a constant, so it does not disturb the sim stream.
   - Default size 64, knob `map_size` 32..128.
   - 9 zone anchors (Base Camp near a corner, zones 1..8 ordered by increasing distance). They are placed on a jittered path or grid, with the boss anchor furthest from camp.
   - A tile's zone is its nearest anchor (Voronoi), so zones are contiguous.
   - Tile kinds: Floor, Grass (encounter tile), Wall (water/rock, blocked), Camp.
   - Grass is about 35% of non-camp, non-boss-grove tiles, in blobs. Walls about 8%, never on anchors.
   - A flood fill from camp turns unreachable floor tiles into Wall. All zones must be reachable and every zone must keep at least 20 Grass tiles.
   - The camp zone has no grass. The King's Grove has no grass: the boss is a visible NPC only.
2. Pathfinding: 4-connected A* with Manhattan heuristic and a deterministic tie-break (lower f, then lower h, then lower y*w+x). No RNG inside A*.
3. Hero goals:
   - At camp: rest, shop, pick target_zone() (existing logic), A* to the nearest reachable tile of that zone.
   - Grind: repeat wander legs. Pick a random Grass tile of the zone within `wander_radius` (default 6) of the hero, A* to it, and walk it.
   - Every step onto a Grass tile rolls `encounter_rate` (default 0.12, knob 0.01..0.5). A win is an encounter fight at that tile.
   - Walk back to camp (A* to the camp tile) when potions==0 and hp<50%, when target_zone changes, or after a flee. Death respawns on the camp tile (existing 25% gold loss).
4. Weighted species tables:
   - `Zone.species` becomes `&[(usize, u32)]` weights. Rule: the first species has weight 4, middle ones 3 and 2, the last 1, for zones with 4 species. Zones with 3 species use 4/3/1. The boss zone has only the boss.
   - Monster level is uniform in the zone range, as today.
5. Roaming elites:
   - Zones 1..7 each get `elites_per_zone` (default 1, knob 0..3) elite NPCs.
   - An elite is a species from the zone table at level zone.max+2, stats x `elite_mult` (default 1.4, knob 1.0..3.0), a distinct NPC id, and a home tile in its zone.
   - Each hero step, an elite does one random-walk step with probability 0.5, constrained to its own zone's non-wall tiles. If the hero is in its zone and within 3 tiles (Manhattan) it steps toward the hero.
   - When the hero and elite are on the same or adjacent tile, an elite fight starts (kind "elite", it can be fled or lost like any fight).
   - A killed elite respawns at home after `elite_respawn_steps` (default 400, derived constant, not a knob) hero steps. Elite XP/gold x2.
   - The boss is an NPC with id `boss` that patrols within 2 tiles of the grove centre. Contact fights it only when the hero's goal is the boss (target zone 8), as today.
6. Monster moves:
   - `Move { name, power: f32, weight: u32, effect: MoveEffect }`.
   - MoveEffect is None | Poison | Drain | Guard | Weaken. It is data only, in data.rs, `MOVES: [&[Move]; 16]` indexed by species. Each species has 2 or 3 moves.
   - On a monster turn (not stunned, not healing), choose a move by weighted rng. Damage uses atk*power. The log text becomes "The <name> uses <Move> ...".
   - The Behavior passives (Aggressive x1.25, charge every 3rd action and so on) are kept as modifiers. Hero skills are unchanged.
   - The move-pick consumes RNG, so the summary stream changes by design.
7. Steps vs the event cap:
   - A step is NOT an event. Per-step events would blow the 65,536 cap.
   - One "walk" event is emitted per leg. It is the new event kind "walk" and carries `path`, the turn-point waypoints [[x,y],...] of the leg (at most 64 points). It also carries the hero's end tile.
   - A "travel" event is kept when the hero first enters a different zone on a leg. Existing zone grouping keeps working.
   - Events per run therefore stay about 1.2x to 2x the old count. The `steps` count is a new Run/Summary field, a counter only.
   - A safety knob `max_steps` (default 5,000,000, range 1000..50,000,000) sets done+capped when reached, so a low encounter_rate cannot loop forever.
   - The lean path counts walk events through emit_lean exactly like full mode.
8. Coordinates in events and the log:
   - HeroSnap gains x,y. MonSnap gains x,y (the encounter tile, or the elite/boss tile).
   - Event gains `npcs: Vec<NpcSnap{id,kind,species,level,x,y}>`, only the NPCs inside the hero's current zone (at most 3), to keep the log small. `id` is a string such as `e3` or `boss`.
   - JSON: hero.x, hero.y, mon.x, mon.y, `npcs`:[{"id","sp","lv","x","y"}], walk events get `"path"`.
   - Old fields are unchanged.
9. Map API:
   - New `GET /api/map?seed=<n>&map_size=<n>` returns `{w,h,tiles:[w-length strings of one char/tile: '.' floor, ',' grass, '#' wall, 'C' camp], zones:[strings of digit 0..8], anchors:[[x,y]x9]}`.
   - It is pure from seed, so the viewer fetches it once per run. The run head also gets `w`, `h` and `steps`.
   - Same Host/limits rules as the other routes.
10. Viewer:
   - A `#map` canvas in the Replay centre column above `#zone`/`#last`. It sizes with fitCanvas to the available box and never scrolls.
   - Bird's-eye mode fits the whole map at an integer pixel size per tile, imageSmoothing off.
   - Tile fills come from new CSS tokens in both themes: `--tile-floor --tile-grass --tile-wall --tile-camp` and `--biome-0..--biome-8` tints.
   - The hero is a distinct accent-coloured marker. NPCs are markers (elite = warn colour, boss = bad colour). The last walk path is drawn as a trail.
   - Detail on demand:
     a. Hover shows a tooltip with tile (x,y), zone name and biome, tile kind and any NPC.
     b. A Zoom toggle button, or a double-click, switches to a 16x16 tile window centred on the hero with large tiles and the species emoji for NPCs.
     c. Zoom state is stored in localStorage `jv-mapzoom`.
   - The data module is viewer/data/map.js, which loads and caches the map.
   - Zone group labels in the log are unchanged. Log lines show "(x,y)" for walk/encounter events.
   - New classes are documented in viewer/STYLE.md.
11. Config knobs (config.rs KNOBS array length and validate() array grow; /api/config lists them; viewer controls are auto-built): map_size, encounter_rate, wander_radius (2..20), elites_per_zone, elite_mult, max_steps.
12. Determinism: same seed and cfg gives byte-identical /api/run JSON and map. The lean summary equals the full summary, including steps.

## Workstreams (dependency order)

### 1. map-core (sonnet)
- Task: sim/src/map.rs with tile kinds, generate(seed,size), zone_of, passable, A* `path(from,to)` and `nearest_tile_of_zone`, the flood-fill reachability fix, and the JSON `map_body`.
- Data tables: weighted zone species and MOVES (data.rs). Registered in lib.rs.
- Config: the 6 new knobs in config.rs (struct, Default, KNOBS, set, validate).
- Files: sim/src/map.rs (new), sim/src/lib.rs, sim/src/data.rs, sim/src/config.rs.
- No sim behaviour change yet, so existing tests must still pass. Zone.species type change is handled with an adapter so sim.rs still compiles.
- Tests that must pass: sim/tests/map_tests.rs, the extended config_tests.
- Verify: `cargo test --manifest-path sim/Cargo.toml` (map and config tests green, nothing else regressed).

### 2. sim-walk (sonnet; depends on 1)
- Task: sim.rs integration.
- Hero x,y, the Goal loop, A* walking, the per-step encounter roll, weighted species picking, elites and boss NPC state with respawn, the monster move picker, walk/travel events with path, NpcSnap per event, the steps counter, max_steps, lean/full parity.
- report.rs: JSON fields (hero.x/y, mon.x/y, npcs, path, head w/h/steps), `run_window_body` unchanged semantics. server.rs: the `/api/map` route and its validation.
- Update the exact-number tests superseded by the new RNG stream, in their own commit with a reason.
- Files: sim/src/sim.rs, sim/src/report.rs, sim/src/server.rs, sim/src/main.rs (steps in batch/summary output if printed), sim/tests/* (only superseded assertions).
- Tests that must pass: sim/tests/walk_tests.rs, map_server_tests.rs, and all existing sim tests (postgame, server, report, config).
- Verify: `cargo test --manifest-path sim/Cargo.toml`. A manual check that `/api/run?seed=1&from=0&count=50` shows x,y and npcs.

### 3. viewer-map (sonnet; depends on 2's JSON shape)
- Task: viewer/data/map.js, the `#map` canvas, tokens in both themes, hover tooltip, zoom toggle, trail, markers, (x,y) in log lines, the STYLE.md doc.
- Files: viewer/index.html, viewer/data/map.js (new), viewer/STYLE.md.
- Tests that must pass: tests/viewer_map_check.mjs, and the existing viewer_*_check.mjs (style, layout, theme, canvas, clip, postgame).
- Verify: `node tests/<each>.mjs`.

### 4. docs (sonnet; depends on 1..3)
- Task: README.md section for the map, hero/monster movesets (hero skill table, monster move table), new knobs, the /api/map route, the event schema additions, the steps-vs-cap note.
- Files: README.md. Verify: the README mentions every new knob key in KNOBS (a test checks this).

## Planned commits (one file each, tests first)
1. test: map generation, A* and config knob tests (sim/tests/map_tests.rs, config_tests.rs)
2. test: walking, encounters, elites, determinism and cap tests (sim/tests/walk_tests.rs)
3. test: map route and JSON schema tests (sim/tests/map_server_tests.rs)
4. test: viewer map checks (tests/viewer_map_check.mjs)
5. feat: weighted species and monster move tables (sim/src/data.rs)
6. feat: map generation and A* (sim/src/map.rs)
7. chore: register the map module (sim/src/lib.rs)
8. feat: map knobs (sim/src/config.rs)
9. feat: walking hero, encounters, elites, monster moves (sim/src/sim.rs)
10. feat: coordinates, npcs and path in run JSON (sim/src/report.rs)
11. feat: /api/map route (sim/src/server.rs)
12. fix: update tests pinned to the old RNG stream (the affected sim/tests files, one commit per file)
13. feat: map data module (viewer/data/map.js)
14. feat: map canvas, tokens, tooltip, zoom (viewer/index.html)
15. docs: map classes (viewer/STYLE.md)
16. docs: map, movesets, knobs and API (README.md)

## Test cases (written first, as failing tests)
Workstream 1, sim/tests/map_tests.rs:
- Same seed and size give an identical map. Different seeds differ.
- All tiles have a zone 0..8. Every zone has at least 20 reachable Grass tiles (zone 0 and 8 excepted: zone 0 has the camp tile, zone 8 has at least 1 reachable floor tile).
- Every non-wall tile is reachable from the camp (flood fill). Wall share is between 3% and 20%.
- Camp zone and the boss grove contain no Grass tiles.
- Zones are 4-connected regions (the Voronoi property).
- A* on an open path: the length equals the Manhattan distance, never crosses walls, never repeats a tile, and is identical on repeat calls. A* to an unreachable or wall target returns None. A* from a tile to itself is empty.
- The map works for sizes 32, 64 and 128.
- Config: the 6 new knobs have defaults, ranges and set/validate rejection of out-of-range or non-numeric values. KNOBS keys match the validate() list. map_size is an integer.
- data: every zone's weights sum > 0, every species has 2 or 3 moves with weight > 0, move power > 0.
Workstream 2, sim/tests/walk_tests.rs and map_server_tests.rs:
- Every event has hero x,y inside the map and on a non-wall tile. Consecutive events' hero positions are within the walk path (a walk event's path starts adjacent to the previous hero tile and ends at the hero tile). Consecutive path waypoints differ in one axis.
- No two consecutive tiles in an expanded path are more than 1 apart.
- Encounter events (kind encounter) have mon.x,y equal to hero.x,y and that tile is Grass.
- Elite events: the elite is adjacent to or on the hero tile, and the species is in the zone table.
- The hero starts at the camp tile and respawn events put the hero back on it. Rest and shop events happen only on the camp tile.
- The hero reaches zone 8's grove in a won run, and the boss fight happens within 2 tiles of the boss NPC.
- Every NPC in `npcs` is inside the map, not on a wall, in the hero's zone, and `npcs` has at most 3 entries.
- The encounter rate: with encounter_rate=0.5 versus 0.05 the number of steps per encounter differs about 10x (ratio between 5 and 20 over a 10-run average). Steps are not events: events.len() <= event_cap and steps > events for default cfg.
- encounter_rate=0.01 with a small max_steps terminates with capped=true. The event cap still truncates at event_cap and capped=true.
- Determinism: the same seed gives identical run_body strings across runs, and across threads (the batch). Lean summary equals the full summary (including steps). Different seeds differ.
- A weighted table check: over 5,000 encounters in zone 1, the first species is picked more than the last one and every species appears.
- A monster move test: encounter logs contain "uses" a move from that species' MOVES; every move of a species is observed in a 20-run sweep; hero skill texts (Power Strike, Brace, Stun, Heal) still appear.
- A post-game test: evolution stage events still fire, max_stage and stage_hist still work with the map.
- JSON schema: hero.x, hero.y, mon.x, mon.y, npcs[], head w,h,steps, and walk events carry a path array of [x,y] pairs, at most 64 points. The json parses as valid JSON (escape check). Windowed `/api/run?from&count` still works.
- `/api/map` returns w,h,tiles,zones with the right dimensions and matches Map::generate. A bad map_size gives 400. A bad Host gives 403. The same seed gives the same body.
Workstream 3, tests/viewer_map_check.mjs (using the existing _viewer_harness.mjs):
- A `#map` canvas exists in the Replay view and its size is within the viewport (no page scroll: document scroll height <= viewport) at 1280x720 and 1920x1080, and 800x600.
- Canvas backing size = css size x DPR and imageSmoothingEnabled is false.
- Bird's-eye shows the whole map (the integer tile size times the map size fits in the canvas).
- The hero marker is drawn at the pixel of the current event's (x,y) (sample pixel colour), and moves when the cursor moves.
- NPC markers are drawn from the event npcs.
- Hover over the canvas shows a tooltip with the coordinates and the zone name.
- The Zoom toggle (and double-click) switches to a 16x16 window centred on the hero (the hero marker stays at the canvas centre) and the state persists via localStorage.
- Both themes: the tile colours are read from tokens and differ between dark and light. The hero marker contrast against the tile is >= 3:1 in both.
- The existing style check still passes (no hex literal, px font or inline style outside the token blocks).
- Missing map or old-format run data (no x,y) degrades gracefully (no exception, the map shows an empty state).
- The log lines for walk/encounter events show "(x,y)".
Workstream 4: README contains each new knob key, `/api/map`, the move tables heading and the hero skills.

## Acceptance criteria
- A won run shows the hero crossing the map zone by zone, grinding on Grass tiles, and returning to the camp tile to rest.
- Hero and NPC (x,y) are present in the events, the log and the API. The map renders crisp in both themes, fits the screen without scrolling, is bird's-eye with hover and zoom detail, and follows STYLE.md.
- The encounter table is weighted. Elites roam visibly and the boss is a placed NPC. Monsters use weighted moves; the hero skills are unchanged.
- Determinism, lean/full parity, the event cap (65,536 default) and the post-game stages are all intact. Steps never become per-step events.
- 6 knobs are present in /api/config and the viewer controls, and are documented. The README is updated.
- `cargo test` passes entirely, and all tests/*.mjs pass.
- Commits are granular, on a feat/ branch, never on main.

## Open questions
None blocking. These defaults can be overridden:
- The map is 64x64 (knob), 4-connected movement, and the walk-event granularity is per leg.
- Elite chase radius 3, and the elite respawn period of 400 hero steps (a constant).
- `npcs` is per-event only for the hero's zone, to keep the log small.
- The species weight pattern is 4/3/2/1.

## Addendum: replay on the map and shared attack animation (new user requirement)
User, verbatim: "replay gotta show the shit on map bro, and on battles you can just use the hero hitting npc or vice versa using some cheap animation (same for all attacks)".
