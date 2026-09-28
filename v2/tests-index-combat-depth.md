# Combat-depth test index

## ws1
# Tests index WS1 (combat rules)

Not compiled or run (cargo is not on the bash PATH here and the APIs do not exist yet; compile errors are expected).

| File | Cases | Notes |
|---|---|---|
| tests/combat_depth_tests.rs (new) | 19 | step-timing, stamina, recovery, punish, stagger, guard, parry, arc, blocked effects, guard break, i-frames, dead |
| tests/status_tests.rs (new) | 19 | stagger, stun DR, club chain, knock, slam, confused, slowed, bleed, poison |
| tests/attacks_data_tests.rs (new) | 8 | move tables, elites, boss, status coverage, frame bounds, log JSON |
| tests/spawn_tests.rs (new) | 2 | 200 seeds; start facing |
| tests/combat_tests.rs (rewritten) | 11 | light dmg 8, cycle 13 (hp 92 after 17 steps, 84 after 18); old block/windup/cycle tests moved to combat_depth |
| tests/physics_tests.rs | edit | `Action::idle()` |
| tests/determinism_tests.rs | +1 | scripted confused/bleed byte-identical |
| tests/fuzz_tests.rs | edit | random attack_kind (0..4), random apply_status, hp/stamina upper bounds, finite |
| tests/common/mod.rs | +`duel_unplaced(seed)` | |

## APIs used (follows handoff Decisions)
`arena_sim::attacks::{Species, Rank, Move, moves}`, `status::Status{ALL, idx}`, `Match::{new_with_setup, apply_status(role, status, dur, mag, source), fx(role)->[u32;7], stacks(role)->[u8;7], set_hp, set_pos, set_facing, fighter(r).state/.facing}`, `Setup{hp,species,rank}`, `matchup::BLOCK_HALF_ARC`, `Action::idle()`, `Action.attack_kind`, `CombatState::{GuardRaise, GuardRecover, Staggered, Stunned, Knocked}`, `Arena::circle_blocked`.
Added conventions: `apply_status(Knock, 6, 2.0, ..)` means magnitude = distance in tiles; Warrior moves[0]=Club, Cleave half_arc 70 deg; boss Slam found by name "Slam"; log fragments `"t":"slam"`, `"t":"dot"`, `"kind":"poison"`, `guard_break`, `parry`.

## Plan issues
- Hero Light total is 13 ticks, plan says bound 14..46; tests use 13..=46.
- Confusion window sign is asserted to alternate between consecutive 15-tick windows (heading +60 then -60); turn inversion asserted on alternate windows (one normal, one inverted).
- Wall-slam test finds the open edge x at runtime instead of hard-coding 47.
- GuardRecover after stamina drain is asserted only as entered (not 8 vs 20); guard-break stun asserted 19..=20.

## ws23
# Test index WS2 and WS3 (combat depth)

Files in v2/sim/tests (APIs as in plans/handoffs/2026-10-23T08-46-32_tests-v2-combat-depth_1.md; shared helpers in tests/common/mod.rs).

| File | Cases | Notes |
|---|---|---|
| sense_tests.rs (updated, `mod common;`) | 12 | N_IN 80, all `sense::idx` consts, fuzz in [-1,1] with random statuses/attack_kind, enemy block 31..=67 zero when unseen (13 tiles, with status and windup), seen Warrior windup shows state/atk one-hot, phase (0,1] and advances, species one-hots, stunned scenario, status fractions linear and mirrored to enemy, IN_RANGE, bias |
| net_tests.rs (updated) | 10 | N_OUT 9, N_HID 24, N_PARAMS = formula = 2745, attack_kind argmax of outputs 6..=8, ties to lowest, determinism |
| genome_ops_tests.rs (updated) | 7 | N_OUT/N_HID layout in W2 test, old-length (1031) and off-by-one lengths rejected with "weights length" |
| fitness_tests.rs (new) | 11 | turtle < no-damage loss < trader, draw < no-damage loss, unseen-guard penalty (seen at 3 tiles gets 0), block-bit toggle no effect, punish cap (5 hits -> 1.8), 1 hit -> 0.6, status reward 2 distinct = 0.3, 6 distinct (Boss Mage) = 0.9, passivity ~0.833, determinism, config defaults |
| screen_tests.rs (new) | 6 | turtle fails, aggressor passes, deterministic, from_dir drops newest turtle and keeps normal (keep=1), all-fail error contains "block_share" and "block_unseen_share", from_dir_opts(..., false) keeps turtle |
| evolution_tests.rs (appended module `species_assignment`) | 4 | species_for deterministic, seed%4 mapping, rank cadence 8/32, boss species Warrior or Mage, counts over 64 seeds (2 boss, 6 elite, 56 grass); existing tests untouched |

## APIs used (beyond existing)
- `arena_sim::attacks::{Species, Rank}` (PartialEq, Debug), `status::Status` (`ALL`, Knock..Poison), `matchup::Match::{new_with_setup, result() -> MatchResult, apply_status(role, Status, dur:u32, mag:f32, source:Role), set_hp(role, f32)}`, `matchup::Outcome` unchanged.
- `Action.attack_kind: u8`; `sense::idx::*`; `net::N_HID`.
- `config.fitness` (FitnessTuning): `draw_penalty`, `punish_reward`, `punish_cap`, `status_reward`, `status_cap` (caps may be int or float; test casts to f32).
- `evolve::score_result(role, &MatchResult, &ArenaConfig) -> f32`; `evolve::terms(...) -> Terms{punish_hits, statuses, punish_reward, status_reward, unseen_guard_share, passivity_share}`.
- `evolve::screen(&Genome, Role, &[Genome], seeds:u64) -> Screen{block_unseen_share, block_share, draw_rate, statuses_per_fight}` (PartialEq, Debug).
- `Library::from_dir` screens by default; `Library::from_dir_opts(dir, role, keep, screen:bool)`; errors list the numbers (contain "block_share" and "block_unseen_share").
- `evolve::species_for(seed) -> (Species, Rank)`.

## Assumptions to watch
- Phase progress: test only requires (0,1] and a change after 2 more ticks, not a direction.
- Guard states for the unseen-guard term: GuardRaise, Blocking, GuardRecover; dist <= 4 and seen is not penalised.
- Punish test relies on `apply_status` Stagger bypassing poise and keeping the victim Staggered; hits at steps 5,18,31,44,57 (5 hits in 68 steps).
- Status-reward tests rely on hits at 1.0 tile in range for Rogue Stab/Dash and Boss Mage Hex/Zap/Slam, all landing on an idle 10000-hp hero.
- Screen aggressor test assumes the plan's panel (idler monster) behaves like the test panel; from_dir's internal panel must also pass the aggressor.
- Compile check done: only missing-API errors (attacks, status, sense::idx, Setup, attack_kind, apply_status, result, set_hp, fitness, score_result/terms, screen, species_for, from_dir_opts). cargo is at ~/.cargo/bin (not on bash PATH).

## ws46
# Tests index WS4 and WS6 (v2/sim/tests)

Placeholders to set by the WS4 implementer from measured numbers: progress_tests.rs BAR_EARLY_A/BAR_EARLY_B (0.65), decisiveness_tests.rs MEASURED_DRAW_RATE (0.10) and MEASURED_UNSEEN_BLOCK (0.02), train_smoke_tests.rs MEASURED_BEAT_RANDOM and MEASURED_BEAT_GEN0 (0.80). Smoke scale (gens/pop) is in each file; check total time under 90 s.

- metrics_tests.rs (6): cells sum to 1, idle pair all unseen/vfar/idle and draw, turtle unseen block share, aggressor seen/close attack + stagger + punish + light share, status credit per role, purity.
  API: `metrics::analyze(&FightLog, Role, Outcome) -> FightMetrics{cells:[[[f32;6];4];2] [seen][dist][action], statuses_applied:[u32;7], punish_hits:u32, parries:u32, attack_kind_share:[f32;3], draw:bool, decisive:bool}`, `block_unseen_share()->f32`. Dist buckets close<=1.2, mid<=4, far<=12, vfar. Actions ATTACK0 BLOCK1 DODGE2 MOVE3 IDLE4 INVOLUNTARY5 (intent bits; involuntary = Staggered/Stunned/Knocked). Uses `matchup::play_with_setup(arena,h,m,seed,&cfg,Setup)`.
- checkpoint_tests.rs (5): header `arena-checkpoint-v2`, v1 resume Err contains "version", v2 round trip, GenStats fields `block_unseen_share, seen_share, statuses_per_fight, punish_per_fight: [f32;2]` and `draw_rate: f32`, JSON keys and CSV header columns (to_csv column count equals header).
- progress_tests.rs (1): 3 seeds, 60 gens pop 24, early 6 and 30, panel from other seeds, mean late-share >= BAR and >= 2 of 3 seeds > 0.5, both roles.
- decisiveness_tests.rs (1): 3 seeds, 40 gens, mean draw rate, mean unseen block share, mean action shares >= 0.02.
- train_smoke_tests.rs (5): jsonl has new keys; deterministic; unknown cmd; fight; mean beat-random and beat-gen0 over 3 seeds; `eval --run D --gens 2 --seeds 2 --out F` JSON (`version`, `gens`, `gen`:2, hero/monster, keys block_unseen_share, block_share, draw_rate, seen_share, statuses_per_fight, punish_per_fight, fights); missing gen -> Err.
- campaign_tests.rs: replay now via `play_with_setup` with `Setup{hp:[max_hp,e.monster_hp],species:[Hero,e.species],rank:[Grass,e.rank]}`; new (5): species/rank by kind (boss Warrior or Mage), determinism, all four species appear, fight_json/run_json carry species/rank/monster_name (as_str), library screen default error contains "block_share" and `Library::from_dir_opts(dir,role,keep,false)` ok, CLI `campaign` errors by default and `--no-screen` succeeds. Encounter gains `species: Species, rank: Rank, monster_name` (String); Species/Rank need Copy, PartialEq, Debug.

Notes: metrics block assumptions (intent-bit classification) are my choice; at 13 tiles the enemy is unseen (range 12). Existing evolution_tests.rs untouched.

## ws5
# WS5 viewer tests index

Run: `cd v2/viewer && npx vitest run`. Currently 11 failed / 56 passed (fx.test.ts fails to import `../src/fx`).
