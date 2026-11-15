# v3 masters round 5: make defence work, then retrain

Read first: plans/2026-10-26_v3-masters-r4-net-collapse.md (goal, rules, acceptance), plans/reports/2026-10-26T10-32-04_impl-v3-masters-r4-l-2/index.md (diagnosis), plans/2026-10-25_v3-masters-overnight.md. Branch feat/v3-mechs. cargo at C:\Users\enthu\.cargo\bin. One file per commit, git add and git commit separate, no Co-Authored-By, tests before implementation.

## Approved by the user's standing goal
Redesign the Parry (and Guard if needed) pose so the parry blade is perpendicular/crossed to the incoming blade and actually deflects, so defence works physically. Teachers must parry/block well (tier_2 share >= 40%). Spam of one move must lose to a varied fighter. No fitness credit for guarding/parrying itself; same rules for both fighters; no external interventions.

## Do (do not stop early; this is a long task; time-box rabbit holes to 30 min each)
1. Failing tests first for the pose geometry (parry angle vs incoming blade) and the 3 failing tests (tier_2 share, teachers_decisive vs skill2, spam loses to tier_2).
2. Implement the redesign. Fix behaviour, do not weaken thresholds without justification.
3. Retrain the three styles (press, counter, duelist), regenerate v3/runs logs plus featured long fights with result.moves, write v3/runs/eval_styles.md with honest numbers (swings per fighter, distinct moves, whiff, punish, capped <5%, kill ticks, win rates, cross-style matchups, spam-vs-style results).
4. Run cargo test --release fully, vitest, tsc; report real pass/fail.
Acceptance: nets use 8+ moves across fights, >=8 swings per fighter per fight, distinct styles, no single-move spam winning, decisive kills.
