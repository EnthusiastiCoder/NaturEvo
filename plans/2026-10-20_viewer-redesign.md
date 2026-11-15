## Goal
Redesign viewer/index.html as a single-viewport "app shell": a bird's-eye summary first, everything else (tuning knobs, batch settings, full event log, extra stats) behind closed-by-default expanders, and NO page scroll at common sizes. Builds on plans/jungle-sim.md and plans/2026-10-20_jungle-sim-run-ui.md (run controls, knobs, log fixes already landed in viewer/index.html). Behaviour does not change; only layout, structure and a summary strip are added.

## Standards and constraints
- User, verbatim: "the most important thing is cognitive load, should be able to get a bird eye view and details when needed can be expanded. everything should fit in screen without scrolling". Earlier complaint: "the layout is ugly as fuck" (wall of ~16 unstyled inputs, truncated seed input, unstyled buttons/tabs, fixed-width chart with a gap, huge empty log, cramped hero panel).
- All workstreams: model sonnet. Branch feat/jungle-sim (never main). One file per commit; `git add` and `git commit` as separate commands; check `git diff` first. Commit messages: no Co-Authored-By (CLAUDE.md overrides the attribution reminder).
- No new dependencies (no CDN, no libs), no build step; emoji placeholders only (no images/icons files).
- file:// mode must still work: run controls (`#run-controls`) stay hidden, no fetch, replay + dashboard work from data/run.js and data/stats.js.
- Keep ALL existing element ids and behaviour: run-controls, seed, run-btn, random-seed, batch-runs, batch-btn, run-status, knob-box, knobs, knobs-reset, knob-<key>, tab-replay, tab-dashboard, replay, play, speed, info, scrub, battle, hbar, hhp, hmeta, zone, memo, mbar, mhp, timeline, log, dashboard, dstats, hist, and `window.__jv {setRun,goto,logIndices}`. `#knob-box` stays a `<details>` (closed by default). Tests tests/viewer_check.mjs, viewer_log_check.mjs, viewer_controls_check.mjs must keep passing, unchanged unless a change is deliberate and noted in the commit.
- Do NOT write layout to fit the old layout; the rules below are the requirement.

## Design (screen regions, top to bottom, all inside one 100dvh grid)
Shell: `html,body{height:100%;margin:0;overflow:hidden}`; `body{display:grid;grid-template-rows:auto auto minmax(0,1fr) auto;height:100dvh}`; no max-width cap that leaves a gap, full width with ~12px padding. Every flex/grid child that may grow has `min-height:0`/`min-width:0`. Scrolling happens only inside bounded panels.

1. Top bar (one row, ~44px, wraps to two rows below 900px): title "Jungle Sim"; tabs Replay | Dashboard (styled segmented control, active state clear); on the right (only when `#run-controls` is shown, i.e. served over http): Seed input (wide enough for a 10-digit seed, `width>=12ch`), Run (primary), Random seed (dice emoji + label), a "Batch" closed `<details id="batch-box">` popover holding `#batch-runs` + `#batch-btn`, a "Tuning" toggle (the `#knob-box` details), `#run-status` text. `#run-controls` remains the wrapper id.
2. Summary strip (Replay tab, ALWAYS visible, the bird's-eye view; ids new): `#summary` containing `#outcome` (badge: WON / LOST / "in progress at event k" derived from run.won and cur), `#sum-level` (hero Lv big), `#sum-hp` (HP bar + "64/64", reuse hbar/hhp inside the strip or mirror), `#sum-zone` (zone emoji + name), `#sum-progress` (progress bar = cur/(N-1) + "event k / N"), and 3 key stats: deaths, gold, potions. Nothing else here. At most ~6 items.
3. Main area (`minmax(0,1fr)`, takes ALL remaining height): left/top = compact battle row (`#battle`: hero emoji + HP bar vs zone vs monster emoji + HP bar, one line ~90px) and the playback row (Play, speed select, `#info`, `#scrub` full width, ONE row); below it `#timeline` canvas fills the remaining space (flex:1, `min-height:80px`, width 100% of container, no fixed pixel width; canvas backing size is set from clientWidth/clientHeight on render and on `resize`, so no gap on the right and no blur). Dashboard tab: `#dstats` as a row of 4 stat cards (win rate large) and `#hist` filling the rest in a bounded box with `overflow:auto`.
4. Detail drawers row (bottom, `auto` height, collapsed = one 36px bar of toggles): `<details id="log-box">` "Event log (n)" containing `#log` and `<details id="more-box">` "More stats" (xp, gold, potions, deaths, final level, monster detail). All closed by default. Opening a drawer must not grow the page: the drawer panel is `position:fixed/absolute` bottom-anchored overlay OR takes height from the timeline row, with `max-height:min(45vh,360px)`; `#log{overflow:auto;height:100%}` scrolls inside. `#knob-box` opens as an overlay popover under the top bar (`position:absolute`, `max-height:calc(100dvh - 120px)`, `overflow:auto`, `#knobs` as a responsive 2-3 column grid of "label above input" cells, each input >= 8ch wide, labels never truncated; Reset knobs button pinned at the top of the panel). Only one drawer open at a time is acceptable (opening one closes the others) to bound height.
Default visible inputs (Replay, served): Seed, speed select, scrub slider only (batch + knobs hidden in closed details). Under file://: only speed select and scrub.
Sizes: required no page scroll (both axes) at 1920x1080, 1366x768, 1280x720 and 800x600, in Replay and Dashboard, with a loaded run, with every drawer closed AND with each drawer open. Below 800 px wide the battle row may stack and top bar wraps; at 800x600 still no page scroll. Below ~640x480 not guaranteed (out of scope), but horizontal page scroll must still not appear.
Style: dark green palette kept; 8px spacing scale; consistent control height 30px; primary button green, secondary buttons outlined; focus rings visible; tabular numbers for stats.

## Workstreams
### A tests-first (already written by the planner's test-writer, see Test cases) 
Failing tests exist before any implementation; implementer must not weaken them.

### B viewer-redesign (sonnet)
Files / commits, in order, one file per commit (all in viewer/index.html, steps are separable):
1. feat: app-shell grid, 100dvh, no body scroll, top bar + tabs styling (CSS + markup).
2. feat: summary strip markup + render() fills `#outcome #sum-level #sum-hp #sum-zone #sum-progress` and stats.
3. feat: compact battle row + playback row; timeline canvas fills container, resizes on window `resize` (use `window.addEventListener("resize")`, no ResizeObserver dependency needed but allowed).
4. feat: drawers (`#log-box`, `#more-box`, `#batch-box`, `#knob-box` as popover) closed by default, inner scroll, one-open-at-a-time; log scroll-to-current only when open (re-run scroll when `toggle` fires).
5. feat: dashboard tab layout (cards row + bounded `#hist`).
6. chore: README viewer screenshot/notes line only if README mentions the viewer layout (optional; separate commit, README.md).
Tests that must pass: tests/viewer_layout_check.mjs (new), tests/viewer_check.mjs, tests/viewer_log_check.mjs, tests/viewer_controls_check.mjs (node), then the browser checklist tests/viewer_layout_checklist.md run via chrome-devtools (read prompts/chrome-devtools-mcp.md first; max 6 tabs; serve with `cargo run --release -- serve --port 8099` from sim/ with PATH including /c/Users/enthu/.cargo/bin, plus open file:// version).
Acceptance: see Acceptance criteria. Verify: `node tests/viewer_layout_check.mjs && node tests/viewer_check.mjs && node tests/viewer_log_check.mjs && node tests/viewer_controls_check.mjs`; then every checklist item with evaluate_script and the screenshots.

## Test cases
Written from the requirements, before implementation:
- tests/viewer_layout_check.mjs (node, static + stub DOM): (1) CSS has `overflow:hidden` on html/body and `height:100dvh|100vh` shell; (2) every panel that holds long content (`#log`, `#knobs`, `#hist`) has `overflow:auto|scroll` and is height-bounded (max-height, or inside a grid/flex area with min-height:0); (3) `#knob-box`, `#log-box`, `#more-box`, `#batch-box` exist as `<details>` with no `open` attribute; (4) `#log` is inside `#log-box`, `#knobs` inside `#knob-box`; (5) no text/number `<input>` is outside a closed details except `#seed` (cognitive-load cap), and `#seed` CSS width >= 12ch; (6) no fixed pixel `width` on `#timeline`/body (no `max-width` < 1600px on body); (7) summary ids exist (`summary outcome sum-level sum-hp sum-zone sum-progress`); (8) behaviour with harness: after `__jv.setRun(run)` and `goto(k)` the summary shows level, `k`, and outcome text WON/LOST consistent with `run.won`; (9) under file:// `#run-controls` stays hidden; (10) all legacy ids listed in Standards still exist.
- tests/viewer_layout_checklist.md (browser, exact evaluate_script snippets and expected values): see the file; includes 4 viewports x 2 tabs x (drawers closed, each drawer open) no-scroll check `document.documentElement.scrollHeight <= window.innerHeight && document.documentElement.scrollWidth <= window.innerWidth`, closed-by-default checks, no clipped inputs (`scrollWidth <= clientWidth` for all inputs and visible labels), inner panels scrollable (`log.scrollHeight > log.clientHeight` after load and scrollTop changes), timeline canvas width equals container width, and screenshots list.
- Existing node tests stay green.

## Acceptance criteria
- Landing view (Replay, drawers closed) shows only: top bar, summary strip, battle+playback row, timeline, drawer toggle bar; no page scrollbar at 1920x1080, 1366x768, 1280x720, 800x600 in both tabs; also with each drawer open.
- Knobs, batch settings, log and more-stats are closed on load and expand on click; open content scrolls inside its own bounded panel.
- Seed input shows a full 10-digit seed; no clipped labels/inputs; timeline fills its container width with no gap; no huge empty area.
- file:// works with controls hidden; served mode run/batch/knobs behaviour unchanged.
- All node tests pass; checklist items all PASS with screenshots saved under research/screenshots/viewer-redesign/ (not committed unless asked).

## Open questions
- Whether "one drawer open at a time" should be enforced (plan assumes yes).
- Dashboard hist as horizontal bars vs vertical columns (plan: keep bars, bounded and scrollable; implementer may switch to columns if it fits better).
