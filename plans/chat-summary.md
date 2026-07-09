# Chat summary – handoff to CTO

## What the user wants
A simulation-style game with little focus on graphics. An autonomous player battles NPC monsters in the jungle and its surroundings and levels up by defeating them. Monsters have fixed behaviors and attributes that scale with their level (Pokemon-like). The sim must run a lot of simulations, very fast, yet not superficial: the whole journey and every moment/action of the main character must be viewable. Build and run it end to end. Sprites: the user chose placeholders only for now.

## Decisions
Full table in `plans/jungle-sim-decisions.md`. In short: Rust sim core + web viewer, replay viewer AND stats dashboard, autonomous hero AI, emoji placeholders, built by the CTO orchestrator.
