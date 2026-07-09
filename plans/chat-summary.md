# Chat summary – handoff to CTO

## What the user wants
A simulation-style game with little focus on graphics. An autonomous player battles NPC monsters in the jungle and its surroundings and levels up by defeating them. Monsters have fixed behaviors and attributes that scale with their level (Pokemon-like). The sim must run a lot of simulations, very fast, yet not superficial: the whole journey and every moment/action of the main character must be viewable. Build and run it end to end. Sprites: the user chose placeholders only for now.

## Decisions
Full table in `plans/jungle-sim-decisions.md`. In short: Rust sim core + web viewer, replay viewer AND stats dashboard, autonomous hero AI, emoji placeholders, built by the CTO orchestrator.

## Design plan
`plans/jungle-sim.md` holds the architecture, rules, commit plan, acceptance criteria and tests. Highlights:
- Seeded deterministic RNG (same seed gives the same journey). Zero-dependency Rust crate in `sim/`.
- 8 monster behaviors (Aggressive, Pack, Defensive, Berserker, Venomous, Trickster, Healer, Charger). 16 species, 9 zones, final boss "Jungle King".
- Hero: XP and levels, skills unlocked by level, potions, weapon/armor shop, death means respawn at camp with a penalty. The hero reacts to behaviors (e.g. Brace or Stun against Chargers).
- CLI: `run --seed N` writes `viewer/data/run.js`; `batch --runs N` writes `viewer/data/stats.js` (threaded).
- Static `viewer/index.html` (works from file://): replay tab (play/pause/speed/scrub, battle panel, log, level timeline) and dashboard tab (aggregate charts). Every event carries a hero state snapshot so any point can be scrubbed to.

## State of the work
- Branch `feat/jungle-sim`. Commit `fb96504` ignores `plans/reports/` and `plans/handoffs/`.
- Written, uncommitted: `sim/Cargo.toml`, `sim/src/rng.rs`, `sim/src/data.rs`. The CTO may reuse or replace them. Not yet written: `sim.rs`, `report.rs`, `main.rs`, the viewer, tuning, the README. Balance (monster scaling versus hero growth) needs tuning via batch runs.
- cto-orchestrator mod is installed at local scope in this workspace.

## Environment and rules
- Windows 11, Windows Editor is Windsurf. Cargo 1.95 lives at `C:\Users\enthu\.cargo\bin` and is not on the bash PATH. Node v24.5.0. `claude.exe` is at `C:\Users\enthu\.local\bin`.
- Follow `CLAUDE.md`: never commit to main, one file per commit, `git add` and `git commit` as separate commands, no Co-Authored-By lines, plans go in `plans/`, research notes in `research/`, ask clarifying questions with AskUserQuestion before starting any task.
- No sprite downloads or sprite agents unless the user later gives permission.
- Run `/introspect` at the end of the session.
