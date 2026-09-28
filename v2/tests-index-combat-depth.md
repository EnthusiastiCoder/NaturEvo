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
