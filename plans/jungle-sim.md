# Jungle Sim – plan

Decisions (from Q&A): Rust sim core + web viewer; replay viewer AND aggregate dashboard; autonomous hero AI; placeholder (emoji) graphics only.

## Architecture
- `sim/` Rust crate, zero dependencies. Seeded deterministic RNG, so the same seed gives the same journey.
  - `data.rs` species, zones, behaviors
  - `sim.rs` combat, hero AI, journey loop, event log
  - `report.rs` JSON writers + batch aggregation (threaded)
  - CLI: `run --seed N` writes `viewer/data/run.js`; `batch --runs N` writes `viewer/data/stats.js`
- `viewer/index.html` static page (works from file://). Replay tab: play/pause/speed/scrub, battle panel, log, level timeline. Dashboard tab: aggregate charts.
