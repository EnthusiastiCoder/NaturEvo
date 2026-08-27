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

Applies to workstream 3 (viewer-map), which now also owns:
- Replay on the map: the map canvas redraws on every cursor change from the current event's hero x,y and npcs (and mon x,y). Play, pause, step, scrubbing (timeline click/drag, seek) and the speed slider all drive it. Walk events animate the hero along `path` waypoints within that event's playback tick. At high speed the animation is skipped and the hero snaps to its tile. No per-step events are added, so the event cap is respected.
- Windowed fetch: the map uses only events in the loaded window, via the existing inWin/fetchWin. If the cursor leaves the window, the map holds the last drawn frame and redraws when the window arrives. It never throws.
- Shared attack animation: one `attackAnim(attackerXY, targetXY)` function. The attacker's marker lunges about 40% of the way toward the target and back, with a short flash ring on the target. It lasts about 250 ms at 1x and shortens or is skipped at high speed. It is the same for every attack kind and both directions (hero to NPC, NPC to hero). Direction comes from the event kind: attack, skill and potion kinds start from the hero. hit_taken starts from the NPC. The attacker flash colour comes from tokens. The target is on the encounter tile (mon x,y).
- Constraints: crisp canvas (DPR aware, no smoothing), tokens for both themes, no new px fonts or hex literals, no scroll. requestAnimationFrame is used only while an animation is active. Animation state is exposed for tests as `window.__mapAnim` = {active, kind, from:[x,y], to:[x,y], t}.

Added test cases (tests/viewer_replay_map_check.mjs, written first, failing):
- Seeking to an event moves the hero marker to that event's (x,y). Seeking backward and forward is consistent.
- Play advances the cursor and the marker. The speed slider changes how fast the marker advances.
- Scrubbing the timeline updates the marker.
- A walk event animates along its path (a mid-animation position lies on a path segment), and at max speed it snaps to the end tile.
- A windowed run (total > window) still replays: crossing a window boundary fetches and the map keeps drawing without errors.
- A hero attack, skill or potion event sets `__mapAnim` with from = hero and to = mon tile. A hit_taken event sets it with from = mon and to = hero. The same animation kind and duration applies to both.
- All attack events use the same animation path (a single code path, same kind field).
- The animation ends (active false) and the marker returns to its tile. It is skipped at the highest speed.
- Canvas stays crisp with no scroll during an animation, in both themes. No console errors.

Updated workstream 3 files and tests: viewer/index.html, viewer/data/map.js, viewer/STYLE.md. Tests: tests/viewer_map_check.mjs, tests/viewer_replay_map_check.mjs, plus the existing viewer checks.
Added commit after commit 14: "feat: replay playback and shared attack animation on map (viewer/index.html)", with a test commit "test: replay on map checks (tests/viewer_replay_map_check.mjs)" placed with the other test commits.

## Addendum 2: fixes from the failed visual review of the built map viewer
Defects found (screenshots in the review scratchpad, s2_/s3_/s5_). Each is a requirement on workstream 3 (viewer-map), which now includes a fix pass:
1. **Map size.** The map canvas must be the main focal element of the Replay view. It is square and as large as the replay area allows, with no page scroll. The bird's-eye view is a whole-map view at an integer tile size of at least 4 css px. Acceptance: the canvas short side is at least 70% of the replay area's short side at 1282x642, 1282x722 and 1540x890 (and at DPR 2). The old 1225x98 box (a ~64px square at 1282x642) is not acceptable. The replay layout puts the map in the centre column's flexible area, and the zone/last text and stats go in side or under strips of fixed small height.
2. **Markers.** Markers are sized in tiles, at least 1.5 tiles across and at least 8 css px at bird's-eye. The hero is an accent-coloured ring plus fill. Elites are diamonds in the warn colour. The boss is a larger crown-style square in the bad colour. Hero and NPC colours are distinct (contrast between the hero and elite fill >= 3:1 in both themes, and the hero ring has >= 3:1 against the tile). A small legend (hero, elite, boss, grass, wall, camp) is shown, using tokens.
3. **Zoom.** The zoomed view is a 16x16 tile window centred on the hero and clamped at the map edges. Tiles are at least 24 css px. NPCs inside the window render with their species emoji. It follows the hero on every event, and is readable in both themes.
4. **First-load map.** The map must be visible on first load. Fetch /api/map?seed&map_size (the server must serve it), with no 404s (`/data/map.js` 404 must be fixed: either serve viewer/data/map.js or inline it) and no console errors. The file:// fallback must still show a map (embed `window.MAP` in the generated run js, or generate client side from the seed is NOT allowed, so ship it in the run data). Run start and window refetch must not blank the canvas.
5. **Attack animation.** A visible lunge plus flash on every attack event, in both directions, as in Addendum 1. Acceptance: during an attack event with the animation on, the pixel at the target tile differs from the idle frame (flash) and the attacker marker's drawn position is offset toward the target at the mid-animation time. `__mapAnim.active` is true. There must be a deterministic test hook: `window.__mapAnimStep(t)` renders the frame at t in [0,1] without timers.
6. **Layout.** At 1282x642 `.main` must not overflow (scrollHeight <= clientHeight) and nothing scrolls. The Event log button must not overlap the timeline y-axis or any other control (bounding boxes do not intersect). The timeline and its controls keep a minimum height and the log button sits in its own toolbar slot.
7. **Dark mode.** Verify via the real app toggle: toggle, then check that the tile colours, markers and legend update, that contrast holds, and that the choice persists in localStorage.

Stale failing tests, decided from the requirements:
- `tests/viewer_canvas_check.mjs` (dpr 1.5 aspect): the requirement is a crisp canvas, meaning backing = css x DPR rounded and no smoothing. An exact aspect ratio at a fractional DPR is not a requirement, since rounding makes it impossible. CORRECT the test to allow the +-1 backing pixel rounding tolerance per axis. Do not delete it.
- `tests/viewer_layout_check.mjs`: the requirement (no scroll, no overlap, fits) is current, so keep it and let the new layout satisfy it. If it hardcodes the old element ids or box sizes that the redesign legitimately moved, CORRECT those assertions to the requirement. Never delete the no-scroll and no-overlap assertions.
- `tests/viewer_polish_check.mjs`: for each failing assertion, keep it if it states a requirement still in STYLE.md or the README (style, theme, controls). Delete only assertions that pin removed markup (the old centre `#zone` box size). Record each deletion with its reason in the commit message.
- Each change is its own commit: "test: correct stale <name> to requirement".

Added tests, written first and failing, in tests/viewer_map_review_check.mjs (new file, using tests/_viewer_harness.mjs):
- Map canvas short side >= 70% of the `#replay` view's short side at 1282x642, 1282x722, 1540x890, and at DPR 1 and 2. The canvas is square (within 1 px).
- Marker size: drawn hero marker >= 8 css px, and sampled pixels at hero and elite centres differ in colour, in both themes. A legend element exists with the 6 entries.
- Zoom: a 16x16 window with tile css size >= 24. Zoom is centred on the hero (the hero pixel is at the canvas centre, except at edges). An NPC in the window is drawn (pixel differs from the tile).
- First load: with no interaction, the map canvas has non-background pixels within 3 s of load. No request returns 404 (record the harness's requests), and no console errors. The file:// mode also shows a map.
- Attack animation: `__mapAnimStep(0.5)` for a hero attack and for a hit_taken event gives a target-pixel change versus idle, and the attacker offset points toward the target.
- Layout: `.main` scrollHeight <= clientHeight and document is not scrollable at 1282x642, 1282x722 and 1540x890. The log button's and timeline's boxes do not intersect, and no two toolbar controls overlap.
- Theme: clicking `#theme-toggle` changes tile and marker colours, keeps contrast >= 3:1 for hero vs tile, and persists.
Commit plan additions: test commits for tests/viewer_map_review_check.mjs and the three stale-test corrections come first. Then these fix commits: "fix: map fills replay area and layout no overflow (viewer/index.html)", "fix: markers, legend, zoom, first-load map and animation hook (viewer/index.html)", and "fix: serve viewer data module (sim/src/server.rs)" if the 404 is a server-side route issue.
Acceptance: all new and corrected viewer tests pass and a re-run of the independent visual review shows no defects 1 to 7.

## Addendum 3: obstacle types and one-step replay movement
User, verbatim: "are there obstacles, like walls, rocks, fences etc? add them, and currently, the replay shows movements very fast, I want a movement be one step".

### Part A: obstacle types (workstream 5, sim-obstacles, sonnet; after workstreams 1 and 2)
Defaults (resolved):
- Blocking tile kinds replace the single Wall: Wall (ruin/temple stone), Rock, Fence, Tree, Water. Chars: '#' wall, 'R' rock, 'F' fence, 'T' tree, 'W' water; non-blocking '.' floor, ',' grass, 'C' camp.
- Biome palette (zone to allowed kinds): 0 camp = Fence ring with a gate; 1 Jungle Edge = Tree, Rock; 2 Dense Jungle = Tree (dense), Rock; 3 River Bank = Water (river band), Rock; 4 Misty Swamp = Water, Tree; 5 Ancient Ruins = Wall, Rock; 6 Bat Caves = Rock, Wall; 7 Serpent Temple = Wall, Fence; 8 Kings Grove = Fence ring (gated), Tree.
- Placement uses the map's own seeded Rng (never the sim stream). Density about 10-14% of non-camp tiles, clustered (blobs, river bands, fence lines).
- Place obstacles first, then flood fill from camp; unreachable open pockets become the biome's obstacle kind; fences always keep a 1-tile gate. A* always finds camp to every zone and to the boss tile. Each zone keeps >= 20 grass tiles; grass is never replaced. No obstacle on anchors, camp tile or boss tile.
- Hero, elites and boss never occupy a blocking tile; A* routes around; encounters unaffected.
- `/api/map` tiles use the chars above and add `legend`: [{"ch","name","blocking"}] (8 entries). Old fields unchanged.
- Viewer: each kind drawn distinctly (rock blob, fence rails and posts, tree canopy, water wave stripes, wall brick lines) with tokens `--tile-rock --tile-fence --tile-tree --tile-water --tile-wall` in both themes; legend entry per kind; tooltip names the kind.
- Placement may change A* lengths and so the RNG stream: pinned-number tests get their own update commit.

Tests first (sim/tests/obstacle_tests.rs, failing):
- Same seed identical map, different seeds differ; only the 8 chars appear.
- All 5 blocking kinds appear across 20 seeds at size 64, each only in an allowed zone.
- Obstacle share 6-18% for sizes 32, 64, 128.
- Over 50 seeds: flood fill from camp reaches every non-blocking tile, every anchor and the boss tile; A* camp to each zone and boss succeeds; >= 20 grass per zone; no obstacle on camp/anchors/boss. Fence rings keep a gate.
- A* never crosses a blocking tile and detours around a hand-built block.
- Over 5 full runs every event's hero, npcs and mon (x,y) is on a non-blocking tile.
- Encounters still fire: steps per encounter within a factor of 2 of 1/encounter_rate.
- /api/map returns only legend chars and a legend of 8 entries with correct blocking flags.
Viewer (tests/viewer_obstacle_check.mjs): each kind has a different pixel colour/pattern at its tile in both themes; legend has 8 entries; tooltip names an obstacle.

### Part B: one tile step per replay tick (workstream 6, viewer-steps)
Unit and defaults:
- One step = one tile move = one replay tick at 1x (existing base 10 ticks/s, about 100 ms). Speed multiplies the tick rate. Each step shows a smooth linear one-tile glide over the tick.
- Replay expands each walk/travel event into one-tile steps through a sub-cursor (event index, step index), one step per tick. Non-walk events take one tick each as today; fight animation as in Addendum 1.
- Elites and boss move one tile per hero step; their per-step positions come from data so they glide in sync.
- Above 8x the glide is skipped (snap per step) but steps still advance one per tick, never a whole path. Scrubbing to an event shows the end tile; scrubbing into a walk's middle uses the sub-cursor. Pause mid-walk holds the step.
- Data (sim steps are not events): walk event JSON carries `steps`: [[x,y],...] (every tile of the leg in order, excluding the start tile) and `nsteps`: one entry per hero step listing [id,x,y] for NPCs in the hero's zone that moved on that step. `path` waypoints stay for compatibility; `npcs` is the end-of-leg state.
- A leg's `steps` is capped at 256 tiles; longer legs split into consecutive walk events. The event cap counts walk events only, not steps. Windowed fetch (count up to 5000) stays valid, with a 2 MB response byte budget: the server trims the window and reports the actual count; the viewer already handles shorter windows.
- Lean mode builds no step lists; step counts unchanged. Same seed gives identical `steps`/`nsteps`.

Tests first (failing):
- sim/tests/steps_tests.rs: first step 4-adjacent to the previous hero tile; consecutive steps differ by 1 in one axis; last step equals hero (x,y); no step on a blocking tile; legs <= 256; sum of step counts equals the head walking-step total; `nsteps` length equals `steps` length, each NPC move is one tile, ids valid; byte-identical JSON for the same seed; events <= event_cap; a 5000-count window returns valid JSON with from/total and at least 1 event within the byte budget.
- tests/viewer_steps_check.mjs: at 1x each tick moves the hero exactly one tile over a 20-tile walk; at 4x four times as many steps per second, still one tile per sub-tick; at <= 8x mid-tick drawn position is strictly between two adjacent tile centres; at 16x it snaps to a centre; elites move one tile per tick in sync; pause freezes and play resumes from the same step; scrubbing into the middle of a walk shows the right step tile; seeking back and forth is consistent; window fetch across a walk boundary neither skips steps nor throws; the drawn tile sequence equals `steps`; crisp canvas, no scroll, both themes; old-format data without `steps` falls back to `path` or a snap with no exception.

### Workstream order (dependency)
5. sim-obstacles (sonnet): sim/src/map.rs, data.rs (palette), sim.rs and report.rs (steps/nsteps, leg splitting), server.rs (legend, window byte budget). After workstreams 1 and 2. Tests: obstacle_tests.rs, steps_tests.rs, existing sim tests. Verify: `cargo test --manifest-path sim/Cargo.toml`.
6. viewer-steps (sonnet): viewer/index.html, viewer/data/map.js, viewer/STYLE.md. Starts only after impl-map-review finishes with viewer/index.html and after 5's JSON shape. Obstacle drawing, tokens, legend, sub-cursor step replay, glide, fallbacks. Tests: viewer_obstacle_check.mjs, viewer_steps_check.mjs and all existing viewer checks.
7. docs (sonnet, last): README.md (obstacle table, legend, steps/nsteps, "one step = one tick", byte budget) and STYLE.md.
Commits (one file each, tests first): the four test files; feat obstacle palette (data.rs); feat obstacle placement and legend (map.rs); feat per-step path and npc steps (sim.rs, report.rs); feat window byte budget and legend (server.rs); update superseded tests (one per file); viewer index.html obstacles; index.html steps and glide; STYLE.md; README.md.
Acceptance: new tests pass; A* routes always exist; no entity on a blocked tile; replay advances exactly one tile per tick at 1x with a smooth glide and speed scales ticks; event cap and windowing hold; distinct obstacles in both themes.

## Addendum 4: permadeath, spacing, visible obstacles
User, verbatim: "let's make death the end of the game for him, just like in real world. and the hero and npcs gotta be farther apart from each other, make the obstacles be visible on the 2D map too bro". Later: "permadeath should be toggle in the config".

### (1) Permadeath (workstream 8, sim-permadeath, sonnet)
- Config field `permadeath: bool`, default true, a real boolean toggle. Knob type boolean in KNOBS (Knob gets a `kind` of number or bool; bool knobs accept only true/false/1/0 and reject anything else). /api/config lists it with kind "bool" and default true. CLI/serve accept `permadeath=false`. /api/run and /api/batch accept `permadeath=true|false|1|0` (400 on junk). The viewer tuning panel (Run controls) shows a toggle switch (role=switch, aria-checked, keyboard operable, token colours, both themes), default ON, sent as a query param on run/batch. Off keeps the old behaviour exactly (respawn at camp, 25 percent gold loss, boss retries up to boss_attempts).
- On: the first hero death (any fight) emits "death" then a final "end" event, then the run stops: no respawn, rest or boss-retry. Run gains `outcome`: won, died or capped. Death before a win gives died, won=false. Death after the boss was beaten ends the run with outcome won and post_deaths 1. deaths is 0 or 1.
- Summary gains `outcome` and `died_level`; run head JSON gains `outcome`; batch stats gain `died`, `died_pct`, death-level histogram, death-zone counts. Viewer: outcome badge (Victory, Fallen, Capped) in the summary strip and post-game panel, dashboard death stats, replay stops at the final event (workstream 9).
- Balance compensation (defaults): potion at hp below 50 percent (40 vs boss); flee an ordinary fight when hp below 25 percent and the monster has over 50 percent hp; rest at camp when hp below 70 percent; retreat when potions are 0 and hp below 70 percent; restock to 5. New knob `zone_margin` default 2 (range 0..6): move up a zone only at level >= zone.min + margin; boss gating level >= boss_level + 3 with 80 percent hp and 5 potions; early xp +20 percent below level 10; skip elites when hero level < elite level - 3. Hero skills unchanged.
- Target band: win rate 15-40 percent (aim 25) over 2000 seeds at defaults. Tuning order: zone_margin, boss gating, flee/heal thresholds, then monster mult defaults. The implementer records final numbers in the README. permadeath=false win rate stays within 5 points of the pre-change value.
Tests first (sim/tests/permadeath_tests.rs, extra cases in config_tests.rs and server_tests.rs):
- On: over 200 seeds each died run has exactly one death event, followed only by an end event, nothing after; deaths <= 1; outcome never won with won=false; won-then-died runs have outcome won.
- Off: respawn events occur and deaths can exceed 1 with monster_atk_mult=3.0; outcome consistent (died when boss_attempts exhausted).
- Outcome, died_level, died, died_pct exist; won+died+capped = runs; lean summary equals full summary; capped alive run gives capped; the 65536 cap and windowing still work; determinism.
- Config: permadeath default true; set accepts true/false/1/0, rejects 2, abc and empty; validate ok; /api/config lists it with kind bool; /api/run?permadeath=false changes behaviour and permadeath=maybe gives 400; /api/batch honours it.
- Balance: 2000-seed batch win rate between 15 and 40 percent; permadeath=false within 5 points of the old value (read the existing pinned value or measure on current code first).
- Viewer (tests/viewer_outcome_check.mjs, tests/viewer_permadeath_toggle_check.mjs): toggle exists in Run controls, default ON, aria-checked reflects state, Space and Enter toggle it, the run request URL contains permadeath=true or false, state persists in localStorage, both themes; badge text per outcome; replay stops at the final event; old data without outcome falls back to the won flag with no exception.

### (2) Spacing (part of workstream 8)
All deterministic from the map seeded Rng.
- Hero starts on the camp tile. Elite homes (zones 1..7) are at least 12 tiles (Manhattan) from camp, the boss home at least 20; any two NPC homes at least 14 apart; at least 6 from a zone border where possible. Retry up to 200 attempts, relaxing minimums 25 percent per 50 failures, never below 8, always a valid reachable tile. Minimums scale with size/64 (min 6) for other sizes. Zone anchors are at least 14 apart at size 64 (scaled).
- Elites roam at most 6 tiles from home and never leave their zone; the boss patrols within 2 tiles. Elite aggro radius is 2. Two NPCs never share a tile and stay at least 4 apart when roaming (a move that would break it is skipped).
- Encounters are not constant: default rate 0.12 plus a 6-step cooldown after any fight, so about 15 steps between fights.
Tests first (sim/tests/spacing_tests.rs): over 50 seeds at sizes 32, 64, 128 the home minimums hold; hero start to NPC homes at least 12 (boss 20); pairwise homes at least 14 and at least 4 across recorded positions (events and nsteps); no NPC leaves its radius or zone or stands on a blocking tile; average steps between encounters over 10 runs is 10 to 25 and no two fights start within 6 steps; determinism.

### (3) Obstacles visible on the map (workstream 9, sonnet, after workstream 8 and impl-viewer-steps)
The implementer confirms each cause in pixel data before changing code. Likely causes: obstacle colours too close to floor, grass or biome tint (under 1.5:1); pattern drawing sub-pixel at small tile size; biome tint overlay drawn after obstacles; map data or legend chars not matching the viewer switch; a token missing in one theme.
- Obstacles are clearly visible in the main map, zoom window and legend swatches in both themes at 1282x642, 1282x722, 1540x890, 1920x1080, DPR 1 and 2.
- Each kind has at least 3:1 contrast against floor, grass and neighbouring tint, and kinds differ pairwise by RGB distance of at least 25. Tokens --tile-rock --tile-fence --tile-tree --tile-water --tile-wall set per theme.
- Draw obstacles last in the tile layer with a darker 1-device-pixel edge. Below 8 css px per tile draw a solid fill plus a darker inner mark; at 8 or more draw the full pattern. Bird's-eye tile at least 5 css px. Legend swatches use the same draw routine.
Tests first (tests/viewer_obstacles_visible_check.mjs, real pixel readback): per viewport and theme (via the real toggle) sample each kind centre pixel against floor, grass and tint pixels (contrast at least 3:1, pairwise distance at least 25); non-floor/grass pixel share 5-25 percent; zoom tiles show at least 2 colours; legend swatch equals the map pixel; an obstacle is never drawn in the floor colour.
