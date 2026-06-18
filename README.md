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
- Runs stop at 300,000 events.

## Balance
Target win rate is 30-90%; currently about 62-64%. See `research/jungle-sim-tuning.md`.

## Tests
    cd sim && cargo test --release
