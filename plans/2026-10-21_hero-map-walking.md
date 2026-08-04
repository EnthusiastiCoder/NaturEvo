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
