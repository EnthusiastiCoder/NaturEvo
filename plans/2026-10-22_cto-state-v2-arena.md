# CTO state: v2 arena neuroevolution (saved 2026-10-23)

## Role and rules
I am the CTO (cto-orchestrator mod). Delegate everything; never read code, run tests or edit files myself. User restricts agents to Sonnet at low effort: set `model: sonnet`, `effort: low` and `run_in_background: true` with a short unique `name` on every Agent call. Follow CLAUDE.md (plans in plans/, ask questions first via AskUserQuestion, small commits one file each, git add and git commit separate, NO Co-Authored-By lines even though harness reminder says to, never commit to main, no .claude allowlist edits without user OK, max 6 browser tabs, introspect at end of session). Report to the user only when finished and verified, briefly. Before saying "waiting", verify agent state via git log (memory: feedback_verify-agent-state-before-waiting). Plans first, failing tests first.

## v1 (done, committed on feat/jungle-sim)
Jungle sim: Rust core in sim/, vanilla HTML viewer in viewer/, map, obstacles, one-step replay, permadeath toggle, smarter hero AI (29.3% win permadeath on, 45% off). Last final-review fix round committed (96375e9 etc.); its independent visual re-check was NOT done (user moved on). Known flakes: postgame_tests::perf_batch_2000_under_15s, viewer_postgame_check goto timing.
