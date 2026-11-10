# CTO state, v3 mechs, second save (2026-10-24)

I am the CTO (cto-orchestrator). I delegate everything and never read code, run tests or edit files. User restricts agents to Sonnet at low effort: every Agent call sets `model: sonnet`, `effort: low`, `run_in_background: true` and a short unique `name`. Context alerts: `checkin_agent` past 100k; `handoff_agent` when projected over 130k, then spawn a successor with "Continue the task described in <path>. Read that file first: it is your full context." (Nearly every agent hits the 130k limit; handoffs are normal. Letting a near-done agent finish is fine.) Earlier state: `plans/2026-10-23_cto-state-v3-mechs.md`.

## User rules (standing)
- Report only when finished and verified, briefly. Tests before implementation. cargo is at `C:\Users\enthu\.cargo\bin`. CLAUDE.md: one file per commit, `git add` and `git commit` as SEPARATE commands, no Co-Authored-By, branch `feat/v3-mechs`.
- Memory saved this segment: `feedback_no_external_intervention_in_fights` (no sudden death, walls, bleed, fatigue timers; fix engagement via learning only). Also memory `feedback_verify-agent-state-before-waiting` updated (3 false "waiting"). Check ListAgents before saying "waiting".
- Stamina is an INTERNAL resource (spent by own actions, fixed regen) and stays. Stalemate pressure is TRAINING-ONLY graded fitness penalty (grows with time without a hit; capped fight = mutual loss). Nothing in a fight changes with elapsed time.
- Fights are deathbattles: continue while both alive; death = torso destroyed or part-health pool below 55%; destroyed head blinds, not KO; per-hit cap 50% of part max.
- Internal mechanics must match what is drawn (sim is the single source of geometry, written into the fight log `geometry` block).
- Run `/introspect` at the end of the session.

## Done and verified (v3, branch feat/v3-mechs)
- Phase 1 vertical slice: WS1 robot, WS2 combat, WS3 duel/log, WS4 evolution, WS6 landing and record, WS7 visual verify. All verifier PASS.
- Dodge credit exploit fixed (would_hit required). Full-length fights (damage cap, pool death). Hitbox/geometry match with viewer (shield plate perpendicular, head 23 cm, neck). Clunkiness diagnosed and reduced (accel-limited motors, action low-pass, W_JERK 8, 1000 gens x pop 64). Sudden death/walls/bleed REMOVED from sim and viewer (user: "cheap").
- Viewer: humanoid look (bigger head, neck, perpendicular shield), impact effects (sparks, rings, hitstop, shake, damage overlays, KO banner, reduced-motion toggle), geometry-driven drawing, 4x speed, block-plate fix (viewer bug at t=595), stalemate note when `result.capped`. 81+ vitest pass. `tests/move-fx.test.ts` fails until WS-E (needs `move-fx.ts`).
- Last verified eval (no-walls run): mean ticks 1394, hits 11.9, death_share 0.911, capped 0.089, win vs earlier champions 0.551, defence 0.43, parry/block/dodge .76/.11/.13. Training took about 12 min per seed (over the 10 min budget): must be brought under 10 min in WS-D.
- Servers: v3 viewer Vite dev server on http://localhost:5174/ (leave it running); 5175 may also still run. Stale v2 shell tasks were killed. The user's other projects' processes must not be touched.

## Sword vocabulary plan (plans/2026-10-23_v3-sword-vocabulary.md)
Index: `plans/reports/2026-10-23T23-02-47_plan-v3-movelib/index.md`. 129 failing tests committed in 9 files (107 Rust, 22 vitest).
- WS-A moveset.rs and executor.rs: DONE, verifier PASS (`plans/reports/2026-10-23T23-29-20_ver-v3-wsa/index.md`).
- WS-B rules.rs: done (16 tests); hooks into combat/duel wired by the WS-C chain (committed by impl-v3-wsc-net-5).
- WS-C net outputs (19), sense inputs, SkillBot tiers, log fields: committed by impl-v3-wsc-net-5 (6 commits). Report `plans/reports/2026-10-24T00-34-48_impl-v3-wsc-net-5/index.md`.
- SkillBot fix chain (running): impl-v3-skillbot-fix, -2, -3 (handoffs under plans/handoffs/, last `2026-10-24T01-03-40_impl-v3-skillbot-fix-3_1.md`), now `impl-v3-skillbot-fix-4` is wrapping up: amend tier-3-vs-tier-1 (measured about 49% free seeds) and tier-2 defence (gap about 0.07) tests with justified thresholds, check guard_break test, run suites, commit, report. Root causes already fixed: 1D obstacle blocking with no jump, permanent Fallen state after destroyed leg. Deviation to confirm reverted: "obstacles do not block walking when a Skill brain is in the match" (user rule: same rules for all). `ACTION_SMOOTH` not applied to executor targets (to justify).
- WS-D (NOT STARTED): grace metrics, curriculum (open against engaging SkillBot tier opponents), 3-seed training, `eval_vocab.md`, within 10 min per seed (currently 12). grace_tests currently do not compile (belong to WS-D). Must include: training-only graded stall penalty already done by impl-v3-nowalls; no guard/block bonuses; defence stays learned.
- WS-E (NOT STARTED): viewer `move-fx.ts` (move callouts, telegraph, OPEN badge, grace panel) to make `tests/move-fx.test.ts` and its fixture pass.
- WS-V: final verifier of the plan. Then browser verify (real Brave via chrome-devtools agent type `claude`, window may be maximized, state size tested).

## Agents running right now
- `impl-v3-posture` (a729ffa70f7a421c6): fixing hunched posture (user screenshot images/14.png: "they dont even stand straight bruh"; torso 25 to 30 degrees forward, head low, stiff legs, low sword V). Diagnose first (ready pose STAND_TARGETS, joint limits, balance, net drive/ACTION_SMOOTH, viewer angle drawing), tests first, no in-fight timers, may add training-side posture cost. Existing runs will need retraining after it.
- `impl-v3-skillbot-fix-4` (a267344578130dd37): wrap-up described above; must re-measure bots after posture changes.
- Both edit v3/sim concurrently; expect minor merge friction.
