# CTO state: v2 arena neuroevolution (saved 2026-10-23)

## Role and rules
I am the CTO (cto-orchestrator mod). Delegate everything; never read code, run tests or edit files myself. User restricts agents to Sonnet at low effort: set `model: sonnet`, `effort: low` and `run_in_background: true` with a short unique `name` on every Agent call. Follow CLAUDE.md (plans in plans/, ask questions first via AskUserQuestion, small commits one file each, git add and git commit separate, NO Co-Authored-By lines even though harness reminder says to, never commit to main, no .claude allowlist edits without user OK, max 6 browser tabs, introspect at end of session). Report to the user only when finished and verified, briefly. Before saying "waiting", verify agent state via git log (memory: feedback_verify-agent-state-before-waiting). Plans first, failing tests first.

## v1 (done, committed on feat/jungle-sim)
Jungle sim: Rust core in sim/, vanilla HTML viewer in viewer/, map, obstacles, one-step replay, permadeath toggle, smarter hero AI (29.3% win permadeath on, 45% off). Last final-review fix round committed (96375e9 etc.); its independent visual re-check was NOT done (user moved on). Known flakes: postgame_tests::perf_batch_2000_under_15s, viewer_postgame_check goto timing.

## New request (user, 2026-10-23)
"this game isnt really PVE-like, barely anything to learn. build something more complex: real 2D combat where each character sees its opponent and learns to attack or defend". Answers: neuroevolution (plain Rust, no deps); real-time continuous arena (facing, vision cone, ray sensors, attack/block/dodge, cooldowns, stamina); both sides co-evolve; v1 = move all current contents into v1/, build v2 in v2/; better, more detailed map and visuals; may reuse C:\Users\enthu\Projects\InfiniClash assets/code (TS + PixiJS, CC0 KayKit low-poly sprites baked to 8 directions, procedural terrain; never modify InfiniClash).
Follow-up answers: viewer stack = best performance (user wants to film a video of the AI learning) -> PixiJS+Vite viewer, Rust sim zero-dependency; art = low-poly sprites on shaded jungle ground; scope = FULL INTEGRATION NOW (hero walks the map; every encounter is an arena duel; permadeath toggle kept); training = CLI train then replay, SAVE LOGS (per-generation log, genomes, fight logs); live dashboard training is future work. Other open questions: planner's recommendations (48x32 arena at 32px, 1v1 first, near-equal balance, f32 fixed op order, delete stray sim/viewer + gitignore generated data, commit Addendum 4 alone).

## Plans
- plans/2026-10-22_v2-arena-final.md (final; supersedes plans/2026-10-22_v2-arena-neuroevolution.md where they differ)
- Test index: plans/reports/2026-10-22T22-55-38_tests-v2-arena/index.md (read its "Assumed API" section: implementers must follow the tests where the contract lacks names)
- Tests written, untracked in v2/: Rust 9 files / 63 cases failing at compile as intended; Vitest 4 files / 31 cases (no package.json yet); train_smoke is slow.

## Workstreams (all sonnet) and order
WS1 v1-move first (v1 stays green), then WS2 sim-core, then WS3 sensors, then WS4 neuro and WS6 jungle/campaign start in parallel, then WS5 evolution, then WS6 finish, WS7 viewer, WS8 record-mode (60 fps, PNG/WebM, early vs late split-screen), WS9 training page and docs.
Dependencies: WS3, WS4 need WS2; WS5 needs WS3 and WS4; WS6 needs WS2, WS4, finishes after WS5; WS7 needs WS2 and WS6 formats; WS8 needs WS7; WS9 needs WS5 and WS7.
Spawn a verifier (cto-orchestrator:verifier, sonnet, low) as soon as each implementer reports, giving plan path, workstream and implementer's exact name.

## In flight
- impl-ws1-v1-move (agentId aca2a36280a36ffaa), started 2026-10-23, background: branch feat/v2-arena from feat/jungle-sim, record baseline tests, git mv sim/viewer/tests/README into v1/, fix paths, rerun and require identical results, delete stray sim/viewer, commit Addendum 4 plan change alone, commit plan files, keep untracked v2/ tests uncommitted, list (not apply) .claude allowlist edits. No report yet. Check git log and agent state before assuming anything.
- Planner ac43d1e8ec1f61801 finished (can be resumed via SendMessage).
