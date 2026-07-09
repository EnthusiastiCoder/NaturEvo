# CTO state, jungle sim (saved before /compact)

I am the CTO (cto-orchestrator). Delegate everything; never read code or run tests myself. Always set `model: sonnet`, `effort: low` on every Agent call (user restriction: Sonnet low effort only, no Opus). Branch `feat/jungle-sim`. Follow CLAUDE.md (one file per commit, `git add` and `git commit` separate, no Co-Authored-By, never commit to main).

## User working rules (also in memory/)
- Tests are written BEFORE implementation, from requirements (planner writes failing tests first). No tests back-filled to match existing code (echo chamber). Finished work gets an independent verifier that looks at real screenshots.
- Report unprompted only when work is DONE and verified, briefly. Give in-progress state only if asked. Do not narrate intermediate steps.
- "open in browser" means just open the page for the user; testing is separate.
- Browser = user's real signed-in Brave (chrome-devtools MCP): localhost and local file only, max 6 tabs, close tabs, no account changes. Read prompts/chrome-devtools-mcp.md first.
- Implementers (type cto-orchestrator:implementer) have NO browser tools; browser/visual checks go to agent type `claude`.
- Run /introspect at end of session (mistakes/2026-10-21_jungle-sim-build.md exists from earlier; new mistakes this session: layout fix sent to implementer without plan or tests; viewer never looked at before user saw it; test-writer tests fitted to existing code; claimed "waiting on B3" when it was finished).
