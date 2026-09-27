## Goal
Base URL (no ?run=, no ?fight=) of v2/viewer shows a LANDING view (run picker + aggregate results), never the demo. ?run= and ?fight= behave exactly as today. Demo only via an explicit "Demo" button. Missing runs give an empty state with exact CLI commands (`train`, `campaign` in v2/sim), not a red banner.

## Standards and constraints
- CLAUDE.md: work on branch feat/v2-arena (never main); small commits, ONE FILE PER COMMIT, `git add` and `git commit` as separate commands, no Co-Authored-By lines (overrides any attribution reminder), check git diff before commit. No DB migrations involved.
- UX rules: cognitive load first, tooltips + info buttons, Lucide icons (inline svg like theme.ts), one CSS token system in v2/viewer/src/style.css, light/dark toggle, accent picker, keyboard operable, no scrolling at 1282x642 up to 1920x1080 (landing panels compact, charts sized to container).
- Dataviz skill (if available) chart guidance: direct labels, few colours from tokens, zero baselines for bars, rates 0-1 axis, colour-blind-safe pairs (hero/monster distinct by colour AND dash), no chart junk.
- Existing facts: main.ts `tryLoadDefault()` fetches `/data/run.json` and falls back to `demoRun()` with a banner; runs live in v2/runs/<name>/ with training.csv (cols gen,avg_ticks,best_hero,best_monster,mean_hero,mean_monster,hof_wr_hero,hof_wr_monster,rnd_wr_hero,rnd_wr_monster,hero/monster_attack/block/dodge), training.jsonl, campaign_on.json / campaign_off.json (keys map,path,encounters[kind,zone,tile,result,fight],outcome), champion_{hero,monster}_gen_N.json, fight_gen_N.json, fights_on/, fights_off/. Some folders are junk (_vv, final, tmp60, *.txt, py): discovery must tolerate missing/odd files.
- Seed/date/permadeath: seed from folder name/first campaign file if present else "n/a"; date from file mtime; permadeath from file name (campaign_on = on, campaign_off = off). Never invent data: missing = "n/a".

## Discovery decision
A small Vite plugin (v2/viewer/vite-plugin-runs.ts, registered in vite.config.ts) serves `GET /api/runs` (index JSON built on each request by scanning v2/runs, one level deep; plus nested seed folders like cd/s1 treated as runs named "cd/s1") and `/runs/<path>` static file passthrough (via existing fs.allow / middleware) so files load without a build step. Pure scanning logic lives in v2/viewer/src/runs-index.ts (`buildIndex(entries)` taking a plain file-listing + file reader abstraction, testable without fs). Why: zero build step, always fresh, no sim change needed. Static build degrades: viewer tries `/api/runs`, then `/runs/index.json` (optional file the plugin also writes at `vite build` via generateBundle/closeBundle copy and the sim CLI may write), else shows empty state with CLI commands. Sim CLI change is optional and out of scope.

## Workstreams (all model sonnet)
Dependencies: WS1 -> WS2 -> WS3 -> WS4. WS1 and the pure parts of WS2 can start in parallel only after tests exist.

### WS1 data-index
Task: runs-index.ts (types RunInfo{name,seed,date,generations,permadeath:{on?:CampaignSummary,off?:CampaignSummary},keyResult,files}, parse training.csv -> rows, parse campaign json -> summary {outcome, wins, losses, encounters, perZone}), vite-plugin-runs.ts, vite.config.ts registration, client `loadIndex(fetchFn)` with /api/runs -> /runs/index.json fallback -> {runs:[],error}.
Files: v2/viewer/src/runs-index.ts, v2/viewer/vite-plugin-runs.ts, v2/viewer/vite.config.ts.
Tests: v2/viewer/tests/runs-index.test.ts.
Acceptance: junk folders skipped without throw; real v2/runs/main yields generations=30 (or its CSV rows) and both permadeath summaries; fetch failure returns empty index with reason.
Verify: `npx vitest run tests/runs-index.test.ts`; `curl localhost:5174/api/runs`.

### WS2 aggregation
Task: pure maths in v2/viewer/src/aggregate.ts: campaignWinRate(on/off with counts k/n), learning curves per generation (hero/monster best fitness, win share vs earlier champions = hof_wr, draw rate = 1 - hero_wr - monster_wr if derivable else omitted with note), actionShares(fights, by visibility {seen,unseen} and distance bucket) computed from loaded fight frames (action, pos, vision fields as available in data.ts Fight type), statuses per fight (via fx.ts statusStats), per-zone win rates from encounters[].zone/result, multi-run pooling. Empty input returns null/zero-count, never NaN.
Files: v2/viewer/src/aggregate.ts.
Tests: v2/viewer/tests/aggregate.test.ts.
Acceptance: known fixtures give exact numbers; n=0 handled; pooling weights by counts not by mean of means.

### WS3 landing-ui
Task: v2/viewer/src/landing.ts + index.html container `#landing` + style.css tokens. Picker table/list (name, seed, date, gens, permadeath on/off, key result), Open button (loads campaign into map view via existing parse path, sets ?run=), duel links (early vs late champions: champion_*_gen first/last -> duel mode), selected-run vs "All runs" toggle, charts as compact inline SVG (win-rate bars with counts, learning curves, action shares, statuses, per-zone), info buttons + tooltips, Lucide icons, keyboard focus order, Demo button, empty state with commands `cargo run --release -- train ...` / `campaign ...` (exact flags read from v2/sim/src/cli.rs and README, not guessed). Layout: CSS grid fits 100vh, no page scroll at 1282x642..1920x1080 (panels scroll internally only if run list is long).
Files: v2/viewer/src/landing.ts, v2/viewer/index.html, v2/viewer/src/style.css (separate commits).
Tests: v2/viewer/tests/landing.test.ts (render to string/DOM-free view-model: empty state text, picker rows, chart specs).
Acceptance: all landing chart/view-model tests pass; no hard-coded colours.
