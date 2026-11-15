## Goal
Build the Jungle Sim end to end per plans/jungle-sim.md, plans/jungle-sim-decisions.md, plans/chat-summary.md (settled; do not re-ask). Rust zero-dep crate in sim/ plus static viewer in viewer/.

## Standards and constraints
- ALL agents (implementers, verifiers, test-writer) must be model "sonnet", low effort. Never Opus.
- Branch feat/jungle-sim (never main). One file per commit; `git add` and `git commit` run as SEPARATE commands; NO Co-Authored-By lines (overrides any attribution reminder). Check `git diff`/file before add. Commit plans first (plans/ files, one per commit), then sim/Cargo.toml, sim/src/rng.rs, sim/src/data.rs (reused as is), etc.
- Cargo is at C:\Users\enthu\.cargo\bin, not on bash PATH: run `export PATH="$PATH:/c/Users/enthu/.cargo/bin"` first (or use full path cargo.exe). Run from sim/ with `cargo ... --manifest-path`.
- Emoji placeholders only; no downloads. Rust crate has zero dependencies (std only, std::thread for batch).
- Acceptance (from plan): `cargo run --release -- batch --runs 2000` finishes in seconds, win rate 30-90%; replay shows every action in order with hero state at any event; same seed gives byte-identical output.
- Existing sim/Cargo.toml, rng.rs, data.rs are REUSED unchanged except: Cargo.toml gains nothing (package jungle-sim => lib crate `jungle_sim`, bin `jungle-sim`). Balance tuning may edit data.rs numbers.

## Fixed API (all workstreams and tests rely on it)
sim/src/lib.rs: `pub mod rng; pub mod data; pub mod sim; pub mod report;`
sim/src/sim.rs:
- `pub struct HeroSnap { pub level:u32, pub hp:i32, pub max_hp:i32, pub xp:u32, pub gold:u32, pub potions:u32, pub zone:usize, pub weapon:usize, pub armor:usize }`
- `pub struct MonSnap { pub species:usize, pub level:u32, pub hp:i32, pub max_hp:i32 }`
- `pub struct Event { pub idx:u32, pub kind:&'static str, pub text:String, pub hero:HeroSnap, pub mon:Option<MonSnap> }`  kinds include: "travel","encounter","attack","skill","hit_taken","potion","level_up","victory_fight","death","respawn","shop","rest","boss","win".
- `pub struct Run { pub seed:u64, pub events:Vec<Event>, pub won:bool, pub deaths:u32, pub final_level:u32 }`
- `pub fn simulate(seed:u64) -> Run` deterministic; hard cap of 300_000 events so it always terminates; hero.hp never < 0 in any snapshot; idx is 0,1,2,... consecutive.
- `pub fn xp_to_next(level:u32) -> u32` strictly increasing for level 1..=60.
- `pub fn monster_stat(base:f32, level:u32) -> f32` non-decreasing in level (Pokemon-like scaling).
sim/src/report.rs:
- `pub fn run_json(run:&Run) -> String` (JS file content: `window.RUN = {...};`), no external crates, strings JSON-escaped.
- `pub struct BatchStats { pub runs:u32, pub wins:u32, pub avg_deaths:f64, pub avg_final_level:f64, pub level_hist:Vec<u32> /*index=level 0..=40*/ }`
- `pub fn batch(runs:u32, threads:usize) -> BatchStats` simulates seeds 0..runs, results independent of thread count.
- `pub fn stats_json(s:&BatchStats) -> String` (`window.STATS = {...};`).
sim/src/main.rs: `run --seed N [--out path]` default viewer/data/run.js; `batch --runs N [--out path]` default viewer/data/stats.js; prints win rate and elapsed. Paths are relative to repo root (create dirs if missing).

## Workstreams (run in order; each depends on the previous; all model sonnet)
### WS1 core-sim (sonnet)
Files: sim/src/lib.rs, sim/src/sim.rs (commit separately: lib.rs, then sim.rs). Implement Hero AI + combat + journey loop per the Fixed API and plans/jungle-sim.md rules: hero starts level 1 at Base Camp; loop: shop gear at camp (buy best affordable weapon/armor, potions at POTION_COST up to 5), travel to highest zone whose min <= level (zone 8 King's Grove only when level >= 32), fight monsters of species from zone (level random in zone min..=max, scaled stats via monster_stat), turn order by spd; 8 behaviors each with a distinct effect (Aggressive: +atk; Pack: spawns 2-3 weak copies as one bundled stronger foe; Defensive: halves damage first 2 turns; Berserker: atk up when hp<50%; Venomous: poison dot; Trickster: 25% dodge; Healer: heals 10%/turn; Charger: big hit every 3rd turn). Hero skills unlock by level (Power Strike L3, Brace L6 used vs Charger, Stun L10 used vs Charger/Berserker, Heal L14). Use potion when hp<35%; retreat to camp to rest and shop when potions=0 and hp<50%; death = respawn at camp, lose 25% gold, deaths++. Win = defeat Jungle King. Every action emits an Event with snapshots.
Tests that must pass: tests/sim_tests.rs (written already).
Verify: `cargo test --test sim_tests` and `cargo build --release`.
Acceptance: determinism, invariants, termination.

### WS2 report-cli (sonnet)
Files: sim/src/report.rs, sim/src/main.rs (separate commits). Implement per Fixed API. Threads via std::thread::scope chunking seeds; aggregate deterministically.
Tests: tests/report_tests.rs.
Verify: cargo test; `cargo run --release -- run --seed 1` twice gives byte-identical viewer/data/run.js (compare with `cmp`); batch --runs 2000 runs in under ~10s.

### WS3 viewer (sonnet)
Files: viewer/index.html (single file, inline CSS/JS, loads data/run.js and data/stats.js via script tags, works from file://, tolerates missing stats.js). Replay tab: play/pause, speed select (1x,4x,16x,64x), range slider scrub over events, battle panel (hero emoji 🧙 + HP bar, monster emoji from species list embedded in the page + HP bar, zone), scrolling log with current line highlighted (click a line to jump), level timeline (canvas line of level vs event idx). Dashboard tab: win rate, avg deaths, avg final level, level histogram bars (canvas/div). Add viewer/data/.gitkeep if data is gitignored-free; commit generated data files NOT required.
Tests: tests/viewer_check.mjs (node script: parses index.html, checks required element ids: tab-replay, tab-dashboard, play, speed, scrub, log, battle, timeline, hist).
Verify: `node tests/viewer_check.mjs` plus a node syntax check of the inline script.

### WS4 tuning-readme (sonnet)
Files: sim/src/data.rs and/or sim/src/sim.rs numbers only (separate commits), README.md. Run batch --runs 2000; adjust monster_stat scaling/gold/XP until win rate within 30-90% (aim ~60%) and avg deaths sane; then write README (build, run, view, rules) and commit. Test: tests/report_tests.rs balance test (`win_rate_sane`, 500 runs, 0.30..=0.90).
Verify: batch output shown, all tests pass.

## Test cases
Placed under sim/tests (Cargo integration tests) plus tests/viewer_check.mjs.
WS1 (sim/tests/sim_tests.rs): same_seed_identical (two simulate(7) runs equal in event count, texts, final snapshots); different_seeds_differ; hp_never_negative_and_le_max across 20 seeds; idx_consecutive; terminates under 300_000 events; xp_to_next_strictly_increasing 1..=60; monster_stat_monotonic for all species bases levels 1..=40; final_level>=1 and events nonempty; every event kind is from the allowed list.
WS2 (sim/tests/report_tests.rs): run_json starts with "window.RUN" and ends with ";" and is byte-identical for same seed; json escapes quotes in text; batch(50,1) == batch(50,4) fields; level_hist sums to runs; stats_json contains "wins"; win_rate_sane (500 runs in 0.30..=0.90) - this one expected to fail until WS4.
WS3: tests/viewer_check.mjs.

## Acceptance criteria
All tests pass; batch 2000 runs completes in seconds with win rate 30-90%; identical bytes for same seed; viewer opens from file:// and scrubs any event showing hero/monster HP and level; commits are one-file, on feat/jungle-sim, no Co-Authored-By.

## Open questions
None.
