# v3 masters round 4: trained nets don't swing

Context: read plans/2026-10-25_v3-masters-overnight.md (goal and acceptance), plans/2026-10-26_cto-state-v3-mechs-3.md (rules), and plans/reports/2026-10-26T10-04-02_impl-v3-masters-r3-l-4/index.md (last round). Branch feat/v3-mechs. cargo at C:\Users\enthu\.cargo\bin. One file per commit, `git add` and `git commit` separate, no Co-Authored-By, tests before implementation.

## Problem
The scripted teachers fight well (median 1078 ticks, 12.1 swings per fighter, 6.2 distinct moves, no capped fights). The trained nets (after GENS=1000) swing about 0.04 times per fight with about 1 distinct move, yet win 80-95% against the teachers. That is not mastery and suggests an exploit (find out how they win: body collision, stagger, knockback, arena edge, stall-then-punish, a bug) or a collapse of the distilled policy (distillation active_acc 0.41 vs 0.85 needed).

## Do
1. Diagnose first. Replay trained-net fights; report how they actually win (cause of death, damage sources, moves per tick) in a short note. Fix any exploit in the sim at its root (same rules for both fighters, no external interventions, no sudden death or timers).
2. Fix distillation so nets reproduce teacher move use (active_acc >= 0.85 test passes, no weakening). Investigate whether evolution then erodes swinging; if so, adjust fitness (landed damage, hits per swing, variety, kill speed credit is allowed; NO credit for merely guarding/blocking/parrying).
3. Also fix the two other failing tests (tier_2 parry/block share, teachers_decisive vs skill2) by fixing behaviour, not by loosening thresholds without justification.
4. Retrain three styles (press, counter, duelist), regenerate v3/runs logs plus featured long fights with result.moves, write v3/runs/eval_styles.md with honest numbers: swings per fighter, distinct moves, whiff rate, punish rate, capped rate (<5%), kill ticks, win rates, cross-style matchups.
5. Run cargo test --release fully and vitest and tsc at the end; report real pass/fail.
Acceptance: nets of each style show 8+ moves used across fights, >=8 swings per fighter per fight, distinct style signatures, capped <5%, decisive kills. If after about 3 hours not reached, report the honest state and the best diagnosis. Do not stop early with PARTIAL; time-box rabbit holes.
Browser screenshots are not your job (a separate agent does them).
