## Goal
Build the Jungle Sim end to end per plans/jungle-sim.md, plans/jungle-sim-decisions.md, plans/chat-summary.md (settled; do not re-ask). Rust zero-dep crate in sim/ plus static viewer in viewer/.

## Standards and constraints
- ALL agents (implementers, verifiers, test-writer) must be model "sonnet", low effort. Never Opus.
- Branch feat/jungle-sim (never main). One file per commit; `git add` and `git commit` run as SEPARATE commands; NO Co-Authored-By lines (overrides any attribution reminder). Check `git diff`/file before add. Commit plans first (plans/ files, one per commit), then sim/Cargo.toml, sim/src/rng.rs, sim/src/data.rs (reused as is), etc.
- Cargo is at C:\Users\enthu\.cargo\bin, not on bash PATH: run `export PATH="$PATH:/c/Users/enthu/.cargo/bin"` first (or use full path cargo.exe). Run from sim/ with `cargo ... --manifest-path`.
- Emoji placeholders only; no downloads. Rust crate has zero dependencies (std only, std::thread for batch).
- Acceptance (from plan): `cargo run --release -- batch --runs 2000` finishes in seconds, win rate 30-90%; replay shows every action in order with hero state at any event; same seed gives byte-identical output.
- Existing sim/Cargo.toml, rng.rs, data.rs are REUSED unchanged except: Cargo.toml gains nothing (package jungle-sim => lib crate `jungle_sim`, bin `jungle-sim`). Balance tuning may edit data.rs numbers.
