# Jungle Sim – plan

Decisions (from Q&A): Rust sim core + web viewer; replay viewer AND aggregate dashboard; autonomous hero AI; placeholder (emoji) graphics only.

## Architecture
- `sim/` Rust crate, zero dependencies. Seeded deterministic RNG, so the same seed gives the same journey.
  - `data.rs` species, zones, behaviors
  - `sim.rs` combat, hero AI, journey loop, event log
  - `report.rs` JSON writers + batch aggregation (threaded)
  - CLI: `run --seed N` writes `viewer/data/run.js`; `batch --runs N` writes `viewer/data/stats.js`
- `viewer/index.html` static page (works from file://). Replay tab: play/pause/speed/scrub, battle panel, log, level timeline. Dashboard tab: aggregate charts.

## Game rules
- Hero levels up via XP. Gear is bought at camp. Skills unlock by level. Potions drop and are bought.
- Monsters are NPCs with fixed behaviors (Aggressive, Pack, Defensive, Berserker, Venomous, Trickster, Healer, Charger). Their stats scale with level.
- Hero reacts to behaviors, e.g. Brace or Stun against Chargers.
- Zones 1-7 by level range, then the boss "Jungle King". Death means respawn at camp with a penalty.

## Commits (small, one file each)
1. plan  2. Cargo.toml + rng  3. data  4. sim  5. report  6. main  7. viewer  8. balance tuning  9. README
