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
