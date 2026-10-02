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

How they fail now: `cargo test --no-run` in v3/sim stops with `can't find lib mech_sim at path v3/sim/src/lib.rs` (cargo is at ~/.cargo/bin, not on PATH in the bash tool). Once src/lib.rs exists, expect unresolved-import errors for the modules below. The tests could not be compile-checked against real code, so minor syntax slips are possible and should be fixed when the first module lands.

## Viewer: v3/viewer/tests (vitest, 46 tests)
Fixture: tests/fixtures/fight_small.json (12 frames t=0..22, 13 parts per fighter, one event of every kind, result win_a, ticks 24).

| File | Cases | Module under test |
|---|---|---|
| parse.test.ts | 5 | src/data (parseFight) |
| camera.test.ts | 6 (3 x 2 sizes) | src/camera (cameraFor) |
| fx.test.ts | 8 | src/fx (hitstopTicks, sparkSpec) |
| damage-style.test.ts | 7 (6 it blocks, one it.each of 2) | src/damage-style (partDamageStyle) |
| hud.test.ts | 7 | src/hud, src/renderer, src/robot-draw |
| landing.test.ts | 5 | src/landing |
| aggregate.test.ts | 3 | src/aggregate |
| record.test.ts | 3 | src/record (planFilmSequence) |
| ui.test.ts | 2 | src/style.css |

How they fail now (verified): `cd v2/viewer; npx vitest run --root ../../v3/viewer` gives 9 failed files. Eight fail with `Cannot find module '../src/<x>'`; ui.test.ts fails with ENOENT on v3/viewer/src/style.css. No syntax errors.

## Invented API (the plan contract is vague; implementers must match or tell the CTO)
Rust:
- robot: Role{Hero,Monster}; Robot::new(role); pub fields x, facing, energy, posture, sword_ang_vel, sword_moving_since, strike_quality, free_regen_time, shield_integrity; Clone; Capsule{a,b:[f32;2],r}; consts DT, STAND_TARGETS, GUARD_TARGETS; Part::ALL, Joint::ALL, Joint::limits()/part(); methods apply_targets (full physics step), fk, capsule, com, total_mass, part_mass, max_hp, hp, hp_frac, set_hp, armour, armour_eff, zone_mult, torque_scale, joint_angle, is_disabled/is_destroyed/is_detached, has_sword/has_shield, shield_active, guard_raised, add_posture, apply_impulse, knock_down, can_act, state() -> State{Idle,Staggered,Fallen,Ko}, is_ko, balance_margin, set_sword, set_shield, damage_part.
- config: IMPACT_CAP, IMPACT_REF, DRAW_SCORE.
- combat: swept_hit, resolve_tick(att, def, &Arena) -> Vec<CombatEvent{Hit(HitEvent{part,point,normal,impact,damage,zone,knockback,hitstop_ticks}),Clash{kind,..},Block{raw,reduced,..}}>, ClashKind{Deflect,Clash,PerfectParry} with log_name(), hitstop_ticks.
- moves: Move (incl KickR/KickL), chain_quality, raw_impact(m_eff,v_n,A,Tm,H,B), health_factor, alignment, balance_factor, required_joints, move_possible.
- arena: Arena::flat/generate, width, obstacles, biome. sense: observe, N_IN, OPP_HP, OPP_POINTS.
- duel: Brain{Net,Dummy,Scripted,Fixed([f32;12])}, Match::new/step/run_to_end/enable_log, fields robots/arena/tick/threats/idle_guard_ticks, MatchResult{outcome,ticks,defence,idle_guard_share,defence_credit}, Outcome{WinA,WinB,Draw}, Resolution, is_live_threat.
- log::fight_json(&Match). fitness: MatchStats, score_stats, ThreatOutcome, defence_credit, zone_weight, score_match. curriculum: Stage, stage_for_gen, hof_enabled, opponents_per_individual. evolve: HallOfFame, TrainConfig::new, train.

TypeScript:
- data.parseFight(json) -> {version, arena, parts, dt, frames, events, result}; throws on version != 3.
- camera.cameraFor(frame, w, h) -> {scale, tx, ty}; screen = (x*scale+tx, ty-y*scale).
- fx.hitstopTicks(events, t); fx.sparkSpec(event) -> null | {kind:'cone'|'radial'|'ring', count, color (0xRRGGBB number), spread, dirAngle}.
- damage-style.partDamageStyle(hp, destroyed) -> {scratches, desaturate, cracks, limp, detached}.
- hud.partHealthColor(hp), hud.hudModel(frame) -> {fighters:[{parts:[{color}], energy, posture, coreHp}]}, hud.defenceTallyModel(def) -> {items:[{key,count,tooltip}]}.
- renderer.rendererOptions(dpr). landing.landingViewModel/renderLandingHtml (run: {name,seed,date,generations,fightGens}; vm {empty, rows[{name,openHref}]}). aggregate.aggregateSeries(number[][]) -> {gens,mean,min,max}; aggregateEval(evals) -> per metric {mean,min,max} (keys defence_rate, idle_guard, draw_share, win_rate_hof). record.planFilmSequence(maxGen) -> [{gen, caption}].
