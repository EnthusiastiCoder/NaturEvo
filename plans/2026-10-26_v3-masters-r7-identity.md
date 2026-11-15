# v3 masters round 7: distinct styles, footwork, no monoculture

Read: plans/reports/visual-review-r6-2026-10-26.md (the visual verdict and top-5 fixes), plans/reports/2026-10-26T13-02-08_impl-v3-masters-r6-l/index.md, v3/runs/eval_styles.md, plans/2026-10-25_v3-masters-overnight.md (acceptance), plans/2026-10-26_cto-state-v3-mechs-3.md (rules). Branch feat/v3-mechs. cargo at C:\Users\enthu\.cargo\bin. One file per commit, git add and git commit separate, no Co-Authored-By, tests before implementation. A verifier may be running cargo test concurrently at the start; if the build lock blocks you, wait.

## Rules (unchanged)
Pure combat deathbattles, same rules for both fighters, no sudden death, walls, bleed, fatigue timers, no in-fight intervention. No fitness credit for merely guarding/blocking/parrying. Credit for landed damage, punish, hits per swing, low jerk, variety, kill speed, spacing/footwork variety is allowed. Stalemate pressure is training-only graded fitness. The sim is the single source of geometry.

## Problem (visual review of the r6 nets)
- DiagUp is 78% (press) and 83% (duelist) of moves; counter is 66% Parry. Kick, LowCut, Backstep, Guard almost never used. Per-fight variety 4.0/2.3/3.7 vs 8+.
- Press and duelist look the same (upright, planted, mirrored swings at 1.7 m).
- Stationary 33-60% of ticks; spacing barely varies.
- Counter: 14 of 50 fights (gens 500-990) are 3001-tick draws, both exhausted and parrying.
- Idle sword rests inside the opponent's body for 30+ tick runs (duelist/999 ticks 1381-1415, 1530-1561); backward-pointing sword 3-30% of ticks.

## Do (do not stop early; time-box rabbit holes to 30 min; main deliverable is retrained nets that look different)
1. Failing tests first for each fix.
2. Fitness: reward move SHARE entropy (not distinct count), landed damage per move type; per-style identity bonuses (press: pressure, forward steps, combos, kicks/low cuts; counter: punish after parry/dodge, backstep-then-strike; duelist: spacing variety, footwork, range control, thrusts and feints). Add footwork/range-variety credit to cut stationary time. Cap repetitive same-move share via training fitness (diminishing returns), not in-fight rules.
3. Counter stalemate: training-only graded fitness so capped fights are mutual losses and counter must convert a parry into a punish; do not reward parry itself.
4. Sim: sword-versus-body (and sword-vs-opponent-limb) collision outside strikes so a resting sword cannot sit inside a body; fix any backward-pointing idle sword pose.
5. Retrain three styles, regenerate v3/runs logs including featured long fights AND make the viewer able to open featured fights (the review found the route only opens fight_gen_N files). Write v3/runs/eval_styles.md with honest numbers: move share per style, per-fight variety, stationary share, spacing spread, capped rate, draw rate, spam-vs-style.
6. Run cargo test --release, vitest, tsc. Report real pass/fail.
Acceptance: per-fight variety 6+ moves; no move above 40% share for any style; styles measurably distinct in move share and spacing; stationary <25%; counter draws <5%; no sword inside body in strikes-free ticks; capped <5%.
