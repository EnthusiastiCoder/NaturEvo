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
Open `viewer/index.html` directly in a browser (file://). Scrub any event to see hero and monster HP and level.

## Rules
- Hero starts at level 1 in Base Camp, shops (best affordable weapon/armor, up to 5 potions), then travels to the highest zone allowed by level.
- Zones: Jungle Edge, Dense Jungle, River Bank, Misty Swamp, Ancient Ruins, Bat Caves, Serpent Temple, and King's Grove (level 32+).
- Monster stats scale with level; turn order by speed. Behaviors: Aggressive, Pack, Defensive, Berserker, Venomous, Trickster, Healer, Charger.
- Hero skills: Power Strike (L3), Brace (L6), Stun (L10), Heal (L14). Potions are used below 35% HP.
- Death respawns at camp with 25% gold lost. Winning means defeating the Jungle King.
- Runs stop at 300,000 events, or as a loss after 8 deaths to the Jungle King.

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

API (GET only): `/api/config`, `/api/run?seed=N&knob=v...`, `/api/batch?runs=N&knob=v...`, plus `/`, `/data/run.js`, `/data/stats.js`.
Caps: binds 127.0.0.1 only; runs 1..=20000; request head max 8 KiB, 5 s read timeout; Host must be 127.0.0.1 or localhost (else 403); bad input gives 400 with `{"error":...}`.

## Balance
Target win rate is 30-90%; currently about 54%. See `research/jungle-sim-tuning.md`.

## Tests
    cd sim && cargo test --release
