# v3 masters round 6: stop chasing the scripted parry rate, get to trained masters

Read: plans/2026-10-26_v3-masters-r5-parry-redesign.md (rules, acceptance), plans/reports/2026-10-26T10-46-59_impl-v3-masters-r5-l-2/index.md, plans/reports/2026-10-26T10-32-04_impl-v3-masters-r4-l-2/index.md. Branch feat/v3-mechs. cargo at C:\Users\enthu\.cargo\bin. One file per commit, git add and git commit separate, no Co-Authored-By, tests before implementation.

## CTO decisions
1. Static parry pose search is saturated (about 24% vs 40%). Stop it. The tier_2 40% parry/block share is a property of a scripted bot reacting to fast attacks; the reaction delay is a designed fairness limit. Re-baseline that test to what a reacting scripted tier can achieve (about 0.2), with the justification written in the test comment and the commit message. This is an explicit, documented decision, not a silent weakening.
2. Dealing with spam: spam of one move must lose through real mechanics, not guard credit: whiffed swings leave the spammer open and the opponent punishes (recovery vulnerability), repeated identical swings get parried/dodged by anticipation. Check what the learned nets can do: the real defence is learned anticipation/dodge/backstep, and the training opponent mix must contain the teachers and spammers so nets learn to counter spam. Keep spamming_one_move_loses_more_hp_to_tier_2 passing or fix the cause.
3. skillbot.rs and duel.rs carry experimental uncommitted edits (retry, telegraph, pressed, pub fields). Evaluate: keep only what helps and is covered by tests (commit each file separately), revert the rest. Leave the tree clean.
4. Fix teachers_are_decisive_against_skill2 (counter 0.55 vs 0.6) by improving the counter teacher behaviour, or re-baseline with justification if it is a pure threshold artifact.

## Then the real work (do not stop early; time-box rabbit holes to 30 min; this is the main deliverable)
Retrain the three styles (press, counter, duelist) with the spam-aware opponent mix, regenerate v3/runs logs plus featured long fights with result.moves, write v3/runs/eval_styles.md with honest numbers (swings per fighter, distinct moves, whiff, punish, capped <5%, kill ticks, win rates, cross-style matchups, spam-vs-style). Run cargo test --release fully, vitest, tsc; report real pass/fail.
Acceptance: nets use 8+ moves across fights, >=8 swings per fighter per fight, distinct styles, no single-move spam winning, decisive kills. If retrain shows a new collapse mode, diagnose it with replays and fix at the root.
