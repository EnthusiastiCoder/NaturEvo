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
