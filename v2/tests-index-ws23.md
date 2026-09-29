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
