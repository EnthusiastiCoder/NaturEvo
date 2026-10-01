# v3 tests index (Phase 1, written before implementation)

Plan: plans/2026-10-23_v3-mechs.md. Nothing is implemented; all tests fail now.

## Rust: v3/sim/tests (crate mech_sim, 117 tests)
Support: v3/sim/Cargo.toml (package mech-sim, lib mech_sim at src/lib.rs, no deps), tests/common/mod.rs (helpers).

| File | Cases | Workstream |
|---|---|---|
| core_determinism_tests.rs | 11 | WS0 rng/genome/net |
| robot_tests.rs | 20 | WS1 |
| balance_tests.rs | 8 | WS1 |
| arena_tests.rs | 3 | WS2 |
| combat_tests.rs | 11 | WS2 |
| moves_tests.rs | 12 | WS2 |
| parry_tests.rs | 6 | WS2 |
| sense_tests.rs | 6 | WS3 |
| duel_tests.rs | 6 | WS3 |
| defence_tests.rs | 6 | WS3 |
| determinism_tests.rs | 3 | WS3 |
| log_tests.rs | 5 | WS3 |
| fuzz_tests.rs | 1 (200 seeds) | WS3 |
| fitness_tests.rs | 10 | WS4 |
| curriculum_tests.rs | 5 | WS4 |
| train_smoke_tests.rs | 3 | WS4 |
| perf_tests.rs | 1 (ignored in debug builds) | WS4 |
