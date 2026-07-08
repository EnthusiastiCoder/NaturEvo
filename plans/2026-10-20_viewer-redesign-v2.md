## Goal
Redesign viewer/index.html into a polished, professional analytics dashboard (shadcn/ui-style: neutral dark theme, restrained palette, KPI cards, tidy charts, compact table, clean toolbar) that gives a bird's-eye view first, expands details on demand, and never scrolls the page. This SUPERSEDES plans/2026-10-20_viewer-redesign.md (v1 layout/regions, workstream split and tests there remain the base; this plan adds and overrides as stated). Context: plans/jungle-sim.md, plans/2026-10-20_jungle-sim-run-ui.md.

## Standards and constraints
User requirements, verbatim:
- "the most important thing is cognitive load, should be able to get a bird eye view and details when needed can be expanded. everything should fit in screen without scrolling"
- "the layout is ugly as fuck" (wall of ~16 unstyled inputs, truncated seed input, unstyled buttons/tabs, fixed-width chart with a gap, huge empty log, cramped hero panel)
- "and have info buttons and tool tip texts, instead of glyphs use proper lucide icons, follow a style format"
- "if needed pull shadcn components or some ui library"
- "I want good ui, not the game ui but the dashboard"
Assumptions (state to user): (1) "dashboard" = professional analytics/observability look, not game chrome; jungle theme survives only as a subtle green accent colour. (2) Hero/monster sprites stay emoji placeholders, small, inside panels (decided earlier); all OTHER icons are Lucide. (3) Dark theme by default (see Open questions). (4) Rust sim core stays zero-dependency.
Process: branch feat/jungle-sim; one file per commit, `git add` and `git commit` separate; check git diff first; NO Co-Authored-By lines (CLAUDE.md overrides). All workstreams sonnet. Tests written first (failing). Keep all legacy ids: run-controls seed run-btn random-seed batch-runs batch-btn run-status knob-box knobs knobs-reset knob-<key> tab-replay tab-dashboard replay play speed info scrub battle hbar hhp hmeta zone memo mbar mhp timeline log dashboard dstats hist, and window.__jv {setRun,goto,logIndices}; `#knob-box` stays a closed `<details>`. file:// must work with run controls hidden. Existing tests tests/viewer_check.mjs, viewer_log_check.mjs, viewer_controls_check.mjs stay green (update only deliberately, noted in commit).
