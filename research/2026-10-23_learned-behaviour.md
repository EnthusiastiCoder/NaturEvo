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

## (2) Block rate seen vs unseen, and by distance

Share of ticks in the Blocking state, matched pairs, 32 fights per gen (n = ticks):

| gen, role | unseen overall | seen overall | unseen far-away (>12) | seen close | seen mid | seen far |
|---|---|---|---|---|---|---|
| 9 hero | 0.21 | 0.03 | 0.22 | 0.00 | 0.06 | 0.11 |
| 9 monster | 0.65 | 0.18 | 0.63 | 0.00 | 0.67 | 0.58 |
| 49 hero | 0.01 | 0.01 | 0.01 | 0.02 | 0.00 | 0.01 |
| 49 monster | 0.01 | 0.00 | 0.01 | 0.00 | 0.01 | 0.01 |
| 99 hero | 0.07 | 0.12 | 0.06 | 0.00 | 0.34 | 0.31 |
| 99 monster | 0.04 | 0.01 | 0.02 | 0.00 | 0.03 | 0.03 |
| 149 hero | 0.22 | 0.05 | 0.24 | 0.01 | 0.14 | 0.15 |
| 149 monster | 0.15 | 0.04 | 0.15 | 0.01 | 0.12 | 0.09 |
| 199 hero | 0.13 | 0.10 | 0.13 | 0.04 | 0.22 | 0.27 |
| 199 monster | 0.12 | 0.05 | 0.12 | 0.01 | 0.17 | 0.16 |

Findings:
- Close range (<= 1.2): blocking is almost never used (0-4 percent) at any gen. Fighters trade blows instead.
- Unseen share of a fight is large: 50-75 percent of all ticks (the duel starts 41 tiles apart, vision range is 12, FOV 120 degrees). Late fights spend about 330 of about 500 ticks closing the distance.
- Block while unseen is the dominant place blocking happens for the weaker or turtling nets (gen 9 monster 0.65, gen 149 hero 0.22). For gen 199 the unseen and seen rates are close (0.12-0.13 vs 0.05-0.10).
- A distinct late-gen habit: PRE-BLOCK while approaching a visible enemy at mid/far range (0.16-0.34 at gens 99/199), then drop it before contact.
- Campaign-style mix (hero 199 vs monster gen G): monster gen 9 blocks 45 percent of all ticks (62 percent of unseen ticks); every other monster gen 19..199 blocks 1-14 percent (gen 59 14, 109 10, 159 12, 199 10 are the highest). `campaign` keeps the 20 newest champions, and grass fights (99 percent of encounters, per `v2/runs/final/campaign.txt`) pick library indices from the oldest 35 percent, i.e. gen 9..79. So the turtle gen 9 monster is in the campaign pool and is the most likely thing the user saw. This is an inference, not verified against the viewer data.

## (3) Dodge

Dodge is small and mostly tied to close range: 2 percent of ticks at gen 9 rising to 4-6 percent mid-run, 3-4 percent at gen 199. By distance (seen, matched pairs): close 3-8 percent, mid/far 0-11 percent (gen 199 hero mid 0.11, far 0.09; monster 0.08/0.04). Unseen: 0-6 percent (gen 199 hero 0.02 far-away, monster 0.03). Dodge is expensive (30 stamina, 45 tick cooldown, `DODGE_STAMINA_COST`, `DODGE_COOLDOWN`) so it cannot be spammed even without the penalty (`-3*(fd-0.25)`), and the population dodge bit decays from 0.5 (gen 0) to 0.05-0.12 (gen 199). No dodge spam in champions. The user's observation is blocking, not dodging.

## (4) Strategies seen

Late champions (gen 199, 32 fights): the dominant strategy is APPROACH AND STRIKE. Mean speed about 2.1-2.2 tiles/s, 58 percent of ticks the distance is shrinking, only 7 percent growing, first contact at tick about 329 of 500, then a toe-to-toe slugfest at distance 0.8 (attack 37-49 percent of ticks). Most fights end 4 hp (one hit) from the winner, i.e. a close trade.

Other strategies found (fight files in the scratchpad, positions in tiles, 30 ticks per second):
- TURTLING / BLOCK-WALK (gen 9). `ws5_final` gen 9 monster, seed 5, 1800 ticks, 98.7 percent in Blocking from tick 0: it holds the block bit, walks at fwd 0.9 (half speed because of the block factor 0.5), never sees the enemy for most of the fight (seen=0 at t=0, 100, 200, 300, 400... even at distance 1.2 in t=500..700 because it is facing away), keeps stamina at 10 and ends 100/100 hp, a draw. Same pattern in `ws5_final` seeds 1, 2, 4, 7 and `final_s3` seeds 3, 5, 8.
- SPINNING WHILE BLOCKING (gen 9 hero, `final_s1` seed 4, ticks 0-480). Stays at about (3.5, 17-19) near its spawn, turn +0.5..+0.9 every tick, blk=1, state Blocking, while the enemy is 40 to 13 tiles away. After the enemy arrives (t about 540) it dodges, then trades blows at distance 0.8 and loses 0-4. This is spawn camping with a spinning shield.
- PRE-BLOCK APPROACH (gen 149, `final_s3` seed 2 hero, 646 ticks). Idle at spawn t=0, then Blocking t=50-300 at fwd 0.99 along the bottom wall (y 20 to 28), drops the block bit about t=300, re-blocks at t=350 (d=15.5, unseen), drops it at t=400 (d=9.2, enemy now seen), meets the enemy at about (31.8, 21) at t=450-500, trades blows at d=0.8 and loses 4 hp short. About 49 percent of ticks in Blocking, mostly while unseen.
- WALL-HUGGING ORBIT / STALEMATE CIRCLING (gen 149, `ws5_final` seed 6, draw 1800 ticks). Both fighters run fwd about 1.0 with strafe +-0.9 and turn +-0.9 (circular strafing). Hero's mean distance to the nearest wall is 1.5 tiles (it runs the border: (47,17) at t=600, (47,9) t=750, (47,4.5) t=1050, (47.4,1.1) t=1500), enemy on a parallel path. Distance between them 10-47, they never meet: first_close = none, 53 percent of ticks approaching and 39 percent receding. Both block with the shield up about 40 percent of the time while orbiting (hero 0.42, monster 0.39), so the "constant blocking from far away" visual is exactly this fight.
- CAMPING: not found as a stable strategy. The only camp-like behaviour is the gen 9 spinner above.
- KITING: no real kiting found. Receding share is 7 percent in late matched fights.
- DODGE SPAM: not found (see 3).
- Also note: monster gen 149 `final_s3` attacks 95 percent of ticks and moves 3 percent (`final_s3` seed 5): a stand-and-swing statue, hits only when the hero comes to it. It beats nothing special but shows attack-bit saturation.

## (5) Root cause from the code

What the rules make free or cheap:
1. Holding block costs nothing. `BLOCK_STAMINA_COST = 15` is charged only when a hit is actually absorbed (`stam_cost[j] += BLOCK_STAMINA_COST`, `matchup.rs` step 4); no per-tick drain while the guard is raised. Stamina regenerates only in the Idle state (step 5), so a blocking fighter does not recover, but stamina matters only for (a) dodging (needs >= 30) and (b) the shield working (`t.stamina > 0.0`). A block-walker that never gets hit never loses anything.
2. Raising and dropping the guard is instant. Idle/Blocking -> Blocking or Windup is decided in the same tick (step 2); an attack from Blocking starts at once with no extra delay. So blocking while walking is a free hedge: the only cost is the 0.5 movement factor (`move_fighter`).
3. The shield covers the vision cone. `BLOCK_HALF_ARC = PI/3` gives 120 degrees, equal to the vision FOV 2.094 rad. It blocks 80 percent of damage (`BLOCK_REDUCTION`) from any hit inside that cone whether the blocker saw the attacker or not.
4. Fitness actively rewards the block bit up to 12 percent of ticks: `fitness += 1.5 * fb.min(0.12) + 0.5 * fa.min(0.3)` (`evolve.rs` `score_match`). `fb` is the share of ticks where the net's block bit was on, regardless of whether an enemy is near, seen, or attacking. That is +0.18 for free. It is small next to a win (4*dealt + 1 + 5*hp share, up to about 10), but it is a gradient right from gen 0 (random nets have the bit on about half the time) and it is the only term that directly trades "block bit" against anything.
5. Damage taken is a linear penalty (`-1.0 * taken`) and `taken` is counted after block reduction, so a perma-blocker that stays out of reach gets the same taken = 0 as a ghost. Draw penalty `DRAW_PENALTY = 3.0` plus `-1.0 * ticks/max_ticks` for not winning makes a pure turtle about -3.8 vs a typical gen 0 best of -3.6: that is why turtling does not take over, but it is not strongly selected against either, and in the early generations (9-19) when few fighters can win, an unkillable blocker is competitive. Its bad fitness is partly masked by the panel of mixed opponents (average over 4 opponents, weighted blend with the worst group).
6. The net default is not "block". With no enemy visible, idle, open field and a fresh recurrent state, the pre-activation `out[4]` (block) is negative for most champions (-0.1 to -1.8), so the default is "walk forward, no attack, no block" (fwd +0.9 to +1.0 everywhere; attack -0.6 to -9). But some champions have a positive default: `final_s3` monster gen 9 +0.23, gen 99 +0.32, gen 199 +0.40; `ws5_final` hero gen 99 +0.92, gen 199 +0.72; `ws5_final` monster gen 9 +0.38; `final_s1` monster gen 199 +0.27. Because own state is an input (inputs 4-10), a few nets latch the block (e.g. `ws5_final` hero gen 199: idle +0.72, already blocking +1.86), others unlatch (gen 99: +0.92 idle, -2.15 blocking). So "block when nothing is visible" is learned, not structural, but the near-zero-input region is exactly where the evolutionary signal is weakest, so whichever constant the genome drifts to stays.
7. Stalemate and timeout rules are weak. The timeout (1800 ticks) is a draw with -3 -1 = -4 for both sides, same as a loss with no damage dealt. The 41 tile start and 12 tile vision range mean first contact only occurs after about 330 ticks anyway, and if one side runs away along a wall (orbit above) the other cannot force contact: no zone shrink, no stamina pressure, no "must engage" rule. Movement penalty for blocking (0.5) is the only thing making a blocker slower than a runner.
8. Selection pressure that reduces blocking exists but is weak: block is almost never used at close range (0-4 percent) because attacking pays (4x dealt) and in a trade the 80 percent reduction applies to the blocker only when it is not attacking (attack takes priority over block in step 2). In the first 20-40 gens the blocking population falls from 0.45 to about 0.2-0.3, and at gen 199 it rebounds (hero s1 0.47, monster s1 0.47, ws5 hero 0.39) which suggests the fitness landscape has no gradient against the leftover block bit once nobody can exploit it.
