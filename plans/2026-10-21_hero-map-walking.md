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
