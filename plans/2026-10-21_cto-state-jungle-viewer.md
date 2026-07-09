# CTO state, jungle sim (saved before /compact)

I am the CTO (cto-orchestrator). Delegate everything; never read code or run tests myself. Always set `model: sonnet`, `effort: low` on every Agent call (user restriction: Sonnet low effort only, no Opus). Branch `feat/jungle-sim`. Follow CLAUDE.md (one file per commit, `git add` and `git commit` separate, no Co-Authored-By, never commit to main).

## User working rules (also in memory/)
- Tests are written BEFORE implementation, from requirements (planner writes failing tests first). No tests back-filled to match existing code (echo chamber). Finished work gets an independent verifier that looks at real screenshots.
- Report unprompted only when work is DONE and verified, briefly. Give in-progress state only if asked. Do not narrate intermediate steps.
- "open in browser" means just open the page for the user; testing is separate.
- Browser = user's real signed-in Brave (chrome-devtools MCP): localhost and local file only, max 6 tabs, close tabs, no account changes. Read prompts/chrome-devtools-mcp.md first.
- Implementers (type cto-orchestrator:implementer) have NO browser tools; browser/visual checks go to agent type `claude`.
- Run /introspect at end of session (mistakes/2026-10-21_jungle-sim-build.md exists from earlier; new mistakes this session: layout fix sent to implementer without plan or tests; viewer never looked at before user saw it; test-writer tests fitted to existing code; claimed "waiting on B3" when it was finished).

## Done and verified
- Sim core, CLI, viewer v1, tuning (53.8% win, batch 2000 = 1076 wins, 108/200). Run-UI plan `plans/2026-10-20_jungle-sim-run-ui.md`: WS A core Config, B report batch, C `serve` (127.0.0.1, std-only), D log fixes, E run controls: all PASS.
- Viewer redesign `plans/2026-10-20_viewer-redesign-v2.md` (addenda 1 to 4): vanilla HTML/CSS/JS, tokens, inline Lucide sprite (24 icons verified exact), dark/light toggle, custom accent, tooltips, STYLE.md. B1 PASS, B2 PASS (visual), B3 done (commits 7ef38e7, d3fa26d etc.), polish list items 1-9 committed (52d9003, 9dff104).
- Post-game plan `plans/2026-10-20_jungle-sim-postgame-evolution.md`: configurable event cap (default 65536, `event_cap` in API), run continues after Jungle King, 5-stage evolution A to E, first_win_event, post stats. WS A (core) PASS, B (report.rs, f5a4479) PASS, C (server-api) PASS. Balance warning from verifier: post-game hero dies about once per kill (avg_post_kills 1515, avg_post_deaths 1572; stage 4 mult 3.0x): consider tuning later (ask planner/implementer; user has not decided).
