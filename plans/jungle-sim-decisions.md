# Jungle Sim – decisions (answered by user, 2026-10-21)

Goal: simulation-style game, little focus on graphics. An autonomous hero battles NPC monsters in the jungle and its surroundings and levels up. Monsters have fixed behaviors and stats that scale with level (Pokemon-like). Many simulations must run very fast without being superficial. The whole journey and every action must be viewable. I build and run it end to end.

| Topic | Decision |
|---|---|
| Stack | Rust sim core + web viewer |
| Viewing | Replay viewer (full per-action event log) AND aggregate stats dashboard |
| Player | Autonomous AI agent: explores, fights, rests, shops, levels |
| Graphics | Emoji/shape placeholders only for now. No downloads, no sprite agents |
| Execution | The CTO orchestrator (cto-orchestrator mod) does the build, not the main session |

## Environment notes
- Branch: `feat/jungle-sim` (never commit to main). Commits are small, one file each, `git add` and `git commit` run separately, no Co-Authored-By lines.
- Cargo is at `C:\Users\enthu\.cargo\bin` (not on bash PATH). Node v24.5.0 is available.
- Detailed plan: `plans/jungle-sim.md`. Starter files already written, uncommitted: `sim/Cargo.toml`, `sim/src/rng.rs`, `sim/src/data.rs`. The CTO may reuse or replace them.
- Mod installed at local scope. Start a new session, then `/cto-align <brief>`, `/cto-handoff`, `go`.
