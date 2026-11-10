# CTO state, v3 mechs, second save (2026-10-24)

I am the CTO (cto-orchestrator). I delegate everything and never read code, run tests or edit files. User restricts agents to Sonnet at low effort: every Agent call sets `model: sonnet`, `effort: low`, `run_in_background: true` and a short unique `name`. Context alerts: `checkin_agent` past 100k; `handoff_agent` when projected over 130k, then spawn a successor with "Continue the task described in <path>. Read that file first: it is your full context." (Nearly every agent hits the 130k limit; handoffs are normal. Letting a near-done agent finish is fine.) Earlier state: `plans/2026-10-23_cto-state-v3-mechs.md`.

## User rules (standing)
- Report only when finished and verified, briefly. Tests before implementation. cargo is at `C:\Users\enthu\.cargo\bin`. CLAUDE.md: one file per commit, `git add` and `git commit` as SEPARATE commands, no Co-Authored-By, branch `feat/v3-mechs`.
- Memory saved this segment: `feedback_no_external_intervention_in_fights` (no sudden death, walls, bleed, fatigue timers; fix engagement via learning only). Also memory `feedback_verify-agent-state-before-waiting` updated (3 false "waiting"). Check ListAgents before saying "waiting".
- Stamina is an INTERNAL resource (spent by own actions, fixed regen) and stays. Stalemate pressure is TRAINING-ONLY graded fitness penalty (grows with time without a hit; capped fight = mutual loss). Nothing in a fight changes with elapsed time.
- Fights are deathbattles: continue while both alive; death = torso destroyed or part-health pool below 55%; destroyed head blinds, not KO; per-hit cap 50% of part max.
- Internal mechanics must match what is drawn (sim is the single source of geometry, written into the fight log `geometry` block).
- Run `/introspect` at the end of the session.
