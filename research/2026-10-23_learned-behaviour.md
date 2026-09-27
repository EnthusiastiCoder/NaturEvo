# What the v2 fighters learned, and why they block with no enemy in sight

Date 2026-10-23. Branch feat/v2-arena. Read-only analysis: no source edited, nothing committed. Analysis scripts and fresh fight JSONs live outside the repo in the session scratchpad. Rule numbers come from `v2/sim/src/matchup.rs` (there is no `combat.rs`; combat is in `matchup.rs`).

## Method and caveats

- Data: saved `fight_gen_N.json` and `training.csv` for `v2/runs/final/s1..s3` and `v2/runs/ws5_final`. Fresh fights came from `arena-sim fight`: matched pairs (hero gen G vs monster gen G) for G in 9, 49, 99, 149, 199, 4 runs x 8 arena seeds = 32 fights per gen. Also hero gen 199 vs every monster gen 9..199 (4 runs x 4 seeds each, a campaign-library style mix).
- Action per tick is taken from the logged combat STATE: Blocking, Dodging, Windup/Active/Recovery = attack, else move (|fwd or strafe| > 0.2) or idle. This is what the viewer draws as a shield. Idle never shows (nets always output some movement), so ignore it.
- "Seen" re-implements `sense::observe` (range 12, cone cos >= 0.5, occlusion by walls, arena regenerated from the seed with a Python port of `world.rs`). Small error: the log stores post-move facing, observation used pre-move facing.
- Distance buckets: close <= 1.2 (attack range), mid 1.2-4, far 4-12, vfar > 12 (out of sensor range, always unseen).
- The `hero_block` columns in `training.csv` are the share of ticks where the net's block BIT is on in the whole population (including while attacking). That is not the same as time spent in the Blocking state, which is much lower.
- Uncertainty: I do not know which fights the user watched (saved gen fights, campaign fights, which run, which gen). The numbers below show that blocking at distance is common in the early gen-9 monsters and moderate in late champions, but "constantly blocking" is only reproduced for gen 9 monsters and a few mid champions.
