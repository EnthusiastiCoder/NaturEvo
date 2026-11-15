# Brief: v3 "two masters of their styles" (overnight, autonomous)

User goal (verbatim): "make sure we have really good fighting styles with the AI, I'm gonna go sleep, when I wake up, I want to see that the AI fights seamlessly and masterfully, it should look like two masters of their styles battling each other. both at the peaks of mastery of their style."

## Standing rules (do not violate)
- Fights are pure combat deathbattles: no sudden death, walls, bleed, fatigue timers or any external intervention. Stamina stays internal. Same rules for both fighters.
- The AI must LEARN defence and grace. No fitness bonus for merely guarding/blocking/parrying. Credit for landed damage, punishes, hits per swing, low jerk, spacing is allowed. Stalemate pressure is training-only graded fitness (capped fight = mutual loss).
- Sim is the single source of geometry; the viewer draws what collides.
- Tests before implementation. Git: branch feat/v3-mechs, ONE file per commit, `git add` and `git commit` as SEPARATE commands, no Co-Authored-By. cargo at C:\Users\enthu\.cargo\bin. cargo test --release, vitest, tsc must pass. Training under ~10 min per seed (find speedups).
- Browser: chrome-devtools MCP on the user's real Brave, max 6 tabs, reuse one tab. Viewer at http://localhost:5174/ (leave running).

## What "done" looks like (acceptance, measured honestly)
1. Distinct fighting STYLES: at least 3 archetypes that emerge from learning pressure and curriculum (not hard-coded in fights), e.g. an aggressive pressure fighter (long combos, advance, punish), a counter-fighter (parry/dodge then riposte, patient spacing), a mobile duelist (distance control, feints, precise single strikes). Each style is a separately co-evolved population (own niche: e.g. fitness-credit weights on landed damage per swing / punish rate / spacing / combo length, seeded from SkillBot tiers) and then cross-evaluated; report a style matrix (win rates, moves used, spacing, hits per swing) proving the styles are distinguishable (behavioural metrics differ significantly).
2. Mastery within a style: high move variety used purposefully (distinct moves >= 8 per fighter over a match), low whiff rate, high punish rate on opponent whiffs, parries leading to ripostes, spacing near own optimal distance, low jerk (smooth), capped/draw share < 5%, fights end in decisive kills, mean length plausible (not 1 hit, not stalling), no both-staggered loops, no idle standing.
3. Seamless motion: smooth transitions between moves, upright posture, no clipping, shield and sword held sensibly (visual review must find no major oddities).
4. Showcase: the viewer opens straight to a featured match between two different-style masters (final generations, a long gripping fight with exchanges, parries, ripostes, a decisive finish), plus a picker to choose style matchups. Record and list the best fight logs.
5. Visual review (agents that LOOK at frames and reason about oddities) after every training round; findings fixed or explicitly reported. Never label an odd-looking pose "intended" without a design source.

## Method
- Measure-first loops: train -> eval metrics -> visual review (browser, frames) -> fix root cause (fitness/curriculum/physics/viewer) -> retrain. Iterate while metrics improve; stop when acceptance is met or after a sensible effort cap, and report honestly what is missed.
- Consider: longer training than 1000 gens if time per seed allows (run overnight is allowed: user is asleep, up to a few hours total), larger population, net capacity (hidden size), sensing (opponent weapon phase/threat direction/distance rates), action smoothing, fitness shaping for combos and counters, hall-of-fame diversity per style, self-play within style then cross-style.
- Every round writes a report in plans/reports/ and updates v3/runs/eval_vocab.md (or eval_styles.md).
