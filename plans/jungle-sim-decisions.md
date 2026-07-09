# Jungle Sim – decisions (answered by user, 2026-10-21)

Goal: simulation-style game, little focus on graphics. An autonomous hero battles NPC monsters in the jungle and its surroundings and levels up. Monsters have fixed behaviors and stats that scale with level (Pokemon-like). Many simulations must run very fast without being superficial. The whole journey and every action must be viewable. I build and run it end to end.

| Topic | Decision |
|---|---|
| Stack | Rust sim core + web viewer |
| Viewing | Replay viewer (full per-action event log) AND aggregate stats dashboard |
| Player | Autonomous AI agent: explores, fights, rests, shops, levels |
| Graphics | Emoji/shape placeholders only for now. No downloads, no sprite agents |
| Execution | The CTO orchestrator (cto-orchestrator mod) does the build, not the main session |
