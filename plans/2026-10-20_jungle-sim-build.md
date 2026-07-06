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
