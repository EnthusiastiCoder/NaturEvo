# v3 vocabulary tests index (plan: plans/2026-10-23_v3-sword-vocabulary.md)

All files are new. The exact invented API is documented in the header comment of each file. Implementers must match it
or change the test header and the test together, reporting the change.

| File | Cases | Workstream | Invented API |
|---|---|---|---|
| v3/sim/tests/moveset_tests.rs | 14 | WS-A | `moveset::{N_MOVES, MoveId (ALL, index, name), MoveParams, Phase, MoveSpec{windup,active,recovery,required,energy}, spec, phase_durations, min_jerk, pose_at}` |
| v3/sim/tests/executor_tests.rs | 21 | WS-A | `executor::Executor{new, step, move_id, phase, progress, committed, whiffed, notify_contact, energy_spent}` |
| v3/sim/tests/rules_tests.rs | 16 | WS-B | `rules::{consts, RuleState{new,tick,begin_recovery,on_parry_success,on_parry_miss,is_open,open_remaining,open_mult,riposte_open,riposte_remaining,guard_broken_remaining}, combine, punish_mult, riposte_windup, guard_break, heavy_block_posture, backstep_counter}` |
| v3/sim/tests/skillbot_tests.rs | 11 | WS-C | `skillbot::SkillBot{new,tier}`, `Brain::Skill(u8)`, test brains `Brain::Spam(MoveId)` and `Brain::Cycle(Vec<MoveId>)`, `Match::move_starts(who)` |
| v3/sim/tests/vocab_duel_tests.rs | 12 | WS-C | log `mv` object and events move/whiff/punish/guard_break/feint/counter/riposte_open, Net brain through Executor, release perf test (cfg not debug) |
| v3/sim/tests/io_tests.rs | 13 | WS-C | `net::{N_OUT=19, MOVE_MARGIN, decode -> Decoded{mv,params,drive}}`, `sense::{OWN_MOVE, OWN_PHASE, OPP_MOVE, OPP_PHASE, MoveInfo, observe_moves}` (existing `observe` keeps its signature) |
| v3/sim/tests/grace_tests.rs | 20 | WS-D | `grace::{whiff_rate, hits_per_swing, punish_rate, move_usage_entropy, tip_jerk, GraceStats, match_grace}`, `curriculum::{OppKind, opponent_mix}`; fitness snapshot and training determinism |
| v3/viewer/tests/move-fx.test.ts | 22 | WS-E | `src/move-fx.ts: moveCallout(events, frame, tNow, dt), phaseStyle(phase, p), openBadge(fighterFrame), hudMoveLine(fighterFrame), graceModel(result, who)`; `parseFight` tolerance for missing `mv` |
| v3/viewer/tests/fixtures/fight_moves.json | fixture | WS-E | 20 frames with `mv`, all 7 new events, `result.moves` {usage, whiff, punish, hits_per_swing, jerk} (per-fighter arrays) |

Total: 129 test cases (107 Rust, 22 vitest).

## How they fail now
- Rust: each file fails to compile with E0432 unresolved import (`mech_sim::moveset`, `executor`, `rules`, `skillbot`, `grace`,
  `net::decode`, `curriculum::OppKind`) or E0599 (`Brain::Skill/Spam/Cycle`, `Match::move_starts`), E0425 (`observe_moves`, `OWN_MOVE`...).
  This is the expected missing-behaviour failure; there are no syntax errors (only resolution errors are reported, further
  type errors can only surface once the API exists).
- Viewer: `npx vitest run tests/move-fx.test.ts` fails on the missing module `../src/move-fx`.
- Guard tests in grace_tests.rs (`fitness_weights_are_unchanged`, `score_stats_has_no_new_term`) would pass on the current
  fitness.rs once the file compiles; they pin the baseline including the W_JERK = 8.0 term added by the motion fix.

## Assumptions to confirm
- Curriculum gens 30..54: only SelfPlay 0.3 and Skill(1) >= 0.4 are pinned (plan says "tier 2 (tier 1 until gen 55)").
- Events on the log carry a flat shape (no nested objects); tests extract `{...}` objects textually.
- Deathbattle: no draws; fuzz bound is `HARD_CAP_TICKS`.
