## Goal
Support runs longer than the first boss kill: a user-settable event cap (default 65536 = 2^16, hard ceiling 1,048,576 = 2^20), a post-game that continues after the Jungle King falls, and monsters that evolve along 5-stage lines (A->B->C->D->E, 4 evolutions). Data flows through run_body/stats_body, the server API and the viewer/dashboard. Builds on plans/jungle-sim.md and plans/2026-10-20_jungle-sim-run-ui.md; tuning history in research/jungle-sim-tuning.md.

## Standards and constraints
- ALL workstreams model sonnet. Branch feat/jungle-sim, never main. One file per commit, `git add` and `git commit` as SEPARATE commands, check `git diff` first. No Co-Authored-By lines (CLAUDE.md overrides any attribution reminder).
- Cargo: `export PATH="$PATH:/c/Users/enthu/.cargo/bin"`, run from `sim/`; std only, zero dependencies. Run `cargo test --release`.
- Determinism: same seed + same config = identical run.
- Do not edit sim/viewer (stray untracked copy).
