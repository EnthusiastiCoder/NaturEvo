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

## In progress when saved
- `impl-b5-polish` (agentId a885c6a7de0433055, resumable via SendMessage): implementing against tests/viewer_polish_check.mjs (35 failing: hero details moved into top card and drawer removed, event log as searchable grouped windowed modal, 20-digit wide seed input w/o spinner, logarithmic speed slider + custom numeric input), tests/viewer_clip_check.mjs and viewer_log_check.mjs (deliberately updated by planner, 1 fail each), tests/viewer_canvas_check.mjs (37 failing: crisp DPR-aware canvases; user screenshot showed blurry level chart). Also earlier queued: histogram nice axis ticks (0/1000/2000) and independent-review defect list (log-scale axis, dashboard sparse/denser, replay chart axes, drawer buttons, scrollbars/slider tracks styling, tooltip stuck, accent popover swatch indicator, favicon 404).
- `planner-redesign` (agentId a3cd9ca519117e920) wrote all viewer tests/addenda; resumable.
- `planner-evolution` (agentId ab49424e58fb86bab) wrote the postgame plan/tests; resumable.

## Next steps
1. When impl-b5 reports: spawn verifier (agent type claude, browser) screenshot-only review at ~1282x642, 1282x722, max viewport (browser caps at 1540x890), both themes, accents, deviceScaleFactor 1, 1.5, 2; check modal log, speed slider, hero details, seed, crisp canvases, histogram ticks; run all tests/*.mjs.
2. Then post-game WS D viewer (windowing, `#stage` badge, `#post-stats`, scrub max 65535, windowed fetch) against tests/viewer_postgame_check.mjs (8 failing). Must run AFTER impl-b5 (same file viewer/index.html). Then verifier.
3. Rebuild release binary and restart the server (`sim/target/release/jungle-sim.exe serve` from repo root, http://127.0.0.1:8080/); a verifier's taskkill probably killed the old server, and the old exe is stale. Use a delegate agent.
4. Post-game balance tuning if user wants (hero death-heavy); user answers needed on planner open questions only if they care (defaults used).
5. Final gate: independent screenshot review; then ask the user before committing untracked plans/, tests/, sim/tests/, research/, mistakes/, viewer/data/run.js, stats.js; stray untracked sim/viewer/ must not be committed (created by batch runs from sim/ cwd).
6. /introspect at the end.
