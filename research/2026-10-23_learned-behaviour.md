# What the v2 fighters learned, and why they block with no enemy in sight

Date 2026-10-23. Branch feat/v2-arena. Read-only analysis: no source edited, nothing committed. Analysis scripts and fresh fight JSONs live outside the repo in the session scratchpad. Rule numbers come from `v2/sim/src/matchup.rs` (there is no `combat.rs`; combat is in `matchup.rs`).

## Method and caveats

- Data: saved `fight_gen_N.json` and `training.csv` for `v2/runs/final/s1..s3` and `v2/runs/ws5_final`. Fresh fights came from `arena-sim fight`: matched pairs (hero gen G vs monster gen G) for G in 9, 49, 99, 149, 199, 4 runs x 8 arena seeds = 32 fights per gen. Also hero gen 199 vs every monster gen 9..199 (4 runs x 4 seeds each, a campaign-library style mix).
- Action per tick is taken from the logged combat STATE: Blocking, Dodging, Windup/Active/Recovery = attack, else move (|fwd or strafe| > 0.2) or idle. This is what the viewer draws as a shield. Idle never shows (nets always output some movement), so ignore it.
- "Seen" re-implements `sense::observe` (range 12, cone cos >= 0.5, occlusion by walls, arena regenerated from the seed with a Python port of `world.rs`). Small error: the log stores post-move facing, observation used pre-move facing.
- Distance buckets: close <= 1.2 (attack range), mid 1.2-4, far 4-12, vfar > 12 (out of sensor range, always unseen).
- The `hero_block` columns in `training.csv` are the share of ticks where the net's block BIT is on in the whole population (including while attacking). That is not the same as time spent in the Blocking state, which is much lower.
- Uncertainty: I do not know which fights the user watched (saved gen fights, campaign fights, which run, which gen). The numbers below show that blocking at distance is common in the early gen-9 monsters and moderate in late champions, but "constantly blocking" is only reproduced for gen 9 monsters and a few mid champions.

## (1) Action shares per generation (share of ticks in each state, 32 matched fights each, both sides pooled by role)

| gen | role | attack | block | dodge | move | mean ticks |
|---|---|---|---|---|---|---|
| 9 | hero | 0.22 | 0.16 | 0.02 | 0.60 | 790 |
| 9 | monster | 0.16 | 0.53 | 0.02 | 0.29 | 790 |
| 49 | hero | 0.30 | 0.01 | 0.06 | 0.63 | 518 |
| 49 | monster | 0.32 | 0.00 | 0.03 | 0.64 | 518 |
| 99 | hero | 0.36 | 0.09 | 0.06 | 0.49 | 573 |
| 99 | monster | 0.48 | 0.03 | 0.03 | 0.46 | 573 |
| 149 | hero | 0.34 | 0.16 | 0.05 | 0.45 | 600 |
| 149 | monster | 0.41 | 0.11 | 0.01 | 0.47 | 600 |
| 199 | hero | 0.37 | 0.12 | 0.04 | 0.47 | 500 |
| 199 | monster | 0.49 | 0.09 | 0.03 | 0.38 | 500 |

Per run the picture is very uneven (block share of ticks, matched pairs): gen 199 hero s1 0.07, s2 0.01, s3 0.11, ws5 0.24; monster s1 0.10, s2 0.02, s3 0.13, ws5 0.09. Gen 9 monster: s3 0.71, ws5_final 0.91; gen 9 hero s1 0.63. Gen 49 is the least blocky generation in every run (0.00-0.03).

Population-level block bit (training.csv, includes ticks where the fighter attacks): gen 0 about 0.45-0.58 (random nets), gen 199 0.13-0.47 (hero s1 0.47, monster s1 0.47, s2 monster 0.41, ws5 hero 0.39). It never went to zero in any run. Attack share stays 0.3-0.5, dodge falls from about 0.5 (gen 0) to 0.05-0.12.

Draws (1800-tick timeout) in matched fights: gen 9 3/32, gen 49 2/32, gen 99 0/32, gen 149 1/32, gen 199 0/32. Late fights end in about 500 ticks (about 17 s).
