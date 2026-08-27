# Jungle Sim

A deterministic Rust simulation of a hero fighting through jungle zones to defeat the Jungle King, plus a static viewer to replay runs.

## Build
Requires Rust (std only, no dependencies).

    cd sim
    cargo build --release

## Run
From the repo root (paths are relative to it):

    cargo run --release --manifest-path sim/Cargo.toml -- run --seed 1      # writes viewer/data/run.js
    cargo run --release --manifest-path sim/Cargo.toml -- batch --runs 2000 # writes viewer/data/stats.js

Same seed gives byte-identical output. Batch results do not depend on thread count.

## View
Open `viewer/index.html` directly in a browser (file://). Scrub any event to see hero and monster HP and level. The map canvas shows the hero, monsters and NPCs; use the zoom control (`jv-mapzoom`) to scale it. Replay plays the walk and fights on the map.

## Rules
- Hero starts at level 1 in Base Camp, shops (best affordable weapon/armor, up to 5 potions), then travels to the highest zone allowed by level.
- Zones: Jungle Edge, Dense Jungle, River Bank, Misty Swamp, Ancient Ruins, Bat Caves, Serpent Temple, and King's Grove (level 32+).
- Monster stats scale with level; turn order by speed. Behaviors: Aggressive, Pack, Defensive, Berserker, Venomous, Trickster, Healer, Charger.
- Hero skills: Power Strike (L3), Brace (L6), Stun (L10), Heal (L14). Potions are used below 35% HP.
- Death respawns at camp with 25% gold lost. Winning means defeating the Jungle King.
- Runs stop at the event cap (default 65,536), or as a loss after 8 deaths to the Jungle King.

## Map and walking
- A seeded tile map (`map_size` per side, default 64) holds the zones; tiles are floor, grass or wall. The hero physically walks between camp and zones, one tile per step, along BFS paths, and wanders up to `wander_radius` tiles in a zone.
- Each step on grass may trigger a random encounter with chance `encounter_rate`.
- NPCs: each zone has `elites_per_zone` roaming elites (stats scaled by `elite_mult`, chase the hero within 3 tiles, respawn after 400 steps) and the Jungle King roams his grove. Touching an NPC starts a fight. Elites and the boss only move while the hero is in their zone.
- Steps are not events: only a leg of walking is emitted as one `walk` event, so the event cap counts events and never individual steps. Steps are counted separately and limited by `max_steps` (safety cap); the run reports `steps`.

### Hero skills (moveset)
Power Strike (L3, 1.8x damage, cooldown 3), Brace (L6, cuts a Charger's mighty charge), Stun (L10, half-damage hit and the foe loses its turn, vs Chargers and Berserkers), Heal (L14, below 50% HP, recovers 30% max HP). Potions heal 50% below 35% HP.

### Monster move tables
Each species has its own move table; a move has a name, power and optional effect (Poison, Drain, Guard, Weaken). Damage is scaled by the move's power relative to the species mean, then by behavior. Moves appear in hit events as "The X uses MOVE on the hero".

## Run from the browser
    cd sim && cargo run --release -- serve [--port 8080] [--root DIR]
Open http://127.0.0.1:8080/ . Run controls (seed, Random seed, Run, batch size, knobs) appear only when served over http; file:// keeps the static replay and dashboard.

Knobs (all optional query params; defaults reproduce the stock sim):

| key | range | default |
|---|---|---|
| monster_hp_mult, monster_atk_mult, boss_hp_mult, boss_atk_mult | 0.1-5.0 | 1.0 |
| xp_mult, gold_mult | 0.1-10.0 | 1.0 |
| hero_hp_base | 10-500 | 50 |
| hero_hp_per_level | 1-60 | 14 |
| hero_atk_per_level | 0.5-12.0 | 3.0 |
| hero_def_per_level | 0.0-8.0 | 1.5 |
| potion_cost | 1-500 | 25 |
| boss_level | 5-40 | 32 |
| boss_attempts | 1-50 | 8 |
| event_cap | 100-1048576 | 65536 |
| postgame_level_cap | 40-200 | 80 |
| evolve_kills | 10-100000 | 300 |
| map_size | 32-128 | 64 |
| encounter_rate | 0.01-0.5 | 0.12 |
| wander_radius | 2-20 | 6 |
| elites_per_zone | 0-3 | 1 |
| elite_mult | 1.0-3.0 | 1.4 |
| max_steps | 1000-50000000 | 5000000 |
| zone_margin | 0-6 | 2 |
| permadeath | true/false (toggle) | true |

API (GET only): `/api/config`, `/api/map?seed=N&map_size=S` (returns `w`, `h`, and `tiles` as rows of `.` floor, `,` grass, `#` wall, plus zone data; map_size 32..=128 else 400), `/api/run?seed=N&knob=v...`, `/api/batch?runs=N&knob=v...`, plus `/`, `/data/run.js`, `/data/stats.js`.
`/api/run` also takes `from` and `count` (count 1..=5000) and returns that slice of events plus `total` and `level_series`; without them a run over 200,000 events is rejected with 400 (use a window).
Caps: event_cap ceiling 1,048,576 (`max_events` in `/api/config`); batch rejected (400) if runs * event_cap > 200,000,000; binds 127.0.0.1 only; runs 1..=20000; request head max 8 KiB, 5 s read timeout; Host must be 127.0.0.1 or localhost (else 403); bad input gives 400 with `{"error":...}`.

Event JSON: every event has `npcs` (visible roaming NPCs: `id`, `sp`, `lv`, `x`, `y`), hero `x`/`y` and monster `x`/`y`; `walk` events also carry `path`, a list of `[x,y]` turn-point waypoints. Run headers include `w`, `h` and `steps`.

Permadeath: with `permadeath=true` (the default) the first hero death ends the run with outcome `died` (`won`, `died` or `capped`). In the viewer it is a switch in the run controls (role=switch, Space or Enter toggles it, default ON, stored in localStorage as `jv-permadeath`); its state is sent as `permadeath=true|false` on every run and batch request, and loading a run that carries a `permadeath` flag sets the switch. The summary badge reads Victory, Fallen or Capped, the replay stops at the final event, and the dashboard shows the died share and the average level at death. Obstacles (wall, rock, fence, tree, water) use high-contrast per-theme tokens with a dark 1 px edge and a pattern, so they stay visible on the 2D map and the zoom in both themes.

## Balance
Target win rate is 30-90%; currently about 54%. See `research/jungle-sim-tuning.md`.

## Tests
    cd sim && cargo test --release
