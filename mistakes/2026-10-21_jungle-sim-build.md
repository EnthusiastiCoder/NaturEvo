# Session Introspection — 2026-10-21

## Task
Build the jungle monster sim game with the cto-orchestrator, using only Sonnet agents at low reasoning effort.

## Mistakes

### 1. Spawned an Opus planner
- **What happened:** The first planner agent was spawned without a model override, so it ran on Opus. The user had said "only sonnet" and objected sharply. I stopped it and respawned `planner-sonnet`.
- **Root cause:** The planner agent type defaults to Opus and I did not set `model` on the Agent call.
- **Impact:** User frustration, a wasted Opus run, and a restarted planner.
- **How to avoid:** When the user restricts models, set `model` and `effort` explicitly on every Agent call. Never rely on an agent type's default.
- **User feedback:** Accurate.

### 2. Overlapping work on WS4 and WS2 verification
- **What happened:** I started the WS4 implementer while the WS2 verifier was still running, so a WS2 fix could have collided with WS4's work. I also ran WS1 alone first without re-reading the plan to see whether WS3 could start in parallel.
- **Root cause:** I relied on the compaction summary's ordering instead of the plan's dependency notes.
- **Impact:** No actual conflict occurred, but WS3 started later than it could have.
- **How to avoid:** Check the plan's dependency section before sequencing workstreams, and flag any overlap to the user.
- **User feedback:** Accurate.

## Unsolved Problems
- The viewer has not been opened in a browser or run against real `run.js`/`stats.js` output. Only `viewer_check.mjs` and field-name checks passed.
- Nothing from `plans/`, `tests/` and `sim/tests/` is committed; those were untracked at the start of the session. A stray untracked `sim/viewer/` directory appeared from a batch run.
