# CTO state 4 (v3 mechs overnight masters) - 2026-10-26

I am the CTO (cto-orchestrator). I delegate everything; I never read code, run tests or edit files. Only file I write is save_plan output. Agent calls: model sonnet, effort low, run_in_background true, short unique name. Branch feat/v3-mechs. cargo at C:\Users\enthu\.cargo\bin.

## Standing rules
- Report only when finished and verified, briefly; status only if asked. Verify with ListAgents before saying "waiting".
- Tests before implementation. One file per commit; git add and git commit as separate commands; NO Co-Authored-By (CLAUDE.md overrides the harness attribution reminder).
- chrome-devtools MCP on user's real Brave: max 6 tabs, reuse one tab, no account actions. Never touch other projects' processes (ClinicalIntel, Algorithms, myclone).
- Fights: pure combat deathbattles, same rules both fighters, no sudden death/walls/bleed/fatigue timers/in-fight intervention. Stamina internal. Stalemate pressure is training-only graded fitness. No fitness credit for guarding/blocking/parrying; credit for landed damage, punish, hits per swing, low jerk, variety, kill speed, spacing is fine. Sim is single source of geometry.
- Phase 2 (tile arena/campaign) only when the user says so. Run /introspect at session end.
- Handoff chain: checkin_agent past 100k; handoff_agent when projected over ~200k; successor prompt "Continue the task described in <path>. Read that file first: it is your full context."
- Implementers keep stopping early with PARTIAL: always say "do not stop early", time-box rabbit holes.

## Done and verified
- r6 (impl-v3-masters-r6-l) verified PASS by ver-v3-masters-r6 (cargo test, vitest 255, tsc, fresh eval matches). Report: plans/reports/2026-10-26T13-05-27_ver-v3-masters-r6/index.md.
- Fixed earlier: parry/guard/backstep sweeps no damage; distillation accuracy 0.966; exploit; mutual stagger; blade solidity; body collision; recoil lean; viewer defects. Tier_2 parry test re-baselined (~0.2) with justification (stated decision).
- r6 numbers: swings per fighter 22/16/27 (press/counter/duelist), capped 0.3/4.8/2.7%, per-fight distinct moves 4.0/2.3/3.7 (target 8+), union 7/5/8. DiagUp spam still wins 21-42% vs nets (verifier: balance item).

## Visual review r6 (plans/reports/visual-review-r6-2026-10-26.md): verdict NOT masters
DiagUp 78% press / 83% duelist, counter 66% Parry; press and duelist look identical; stationary 33-60%; counter 14/50 draws; idle sword inside opponent body 30+ ticks; backward sword 3-30%; viewer route only opens fight_gen_N (not featured fights). No facing-away, no frozen binds, lean fine.
