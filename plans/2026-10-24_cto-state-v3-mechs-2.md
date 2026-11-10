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
