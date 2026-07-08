## Goal
Support runs longer than the first boss kill: a user-settable event cap (default 65536 = 2^16, hard ceiling 1,048,576 = 2^20), a post-game that continues after the Jungle King falls, and monsters that evolve along 5-stage lines (A->B->C->D->E, 4 evolutions). Data flows through run_body/stats_body, the server API and the viewer/dashboard. Builds on plans/jungle-sim.md and plans/2026-10-20_jungle-sim-run-ui.md; tuning history in research/jungle-sim-tuning.md.

## Standards and constraints
- ALL workstreams model sonnet. Branch feat/jungle-sim, never main. One file per commit, `git add` and `git commit` as SEPARATE commands, check `git diff` first. No Co-Authored-By lines (CLAUDE.md overrides any attribution reminder).
- Cargo: `export PATH="$PATH:/c/Users/enthu/.cargo/bin"`, run from `sim/`; std only, zero dependencies. Run `cargo test --release`.
- Determinism: same seed + same config = identical run.
- Do not edit sim/viewer (stray untracked copy).

### Assumptions (user's words: "custom event cap (optional for me to set, by default 2^16) and yes it keeps playing boss after win, enemies keep getting stronger or get evolved ... A->B->C->D->E")
1. New knob `event_cap` (u32), default 65536, range 100..=1_048_576 (const `config::MAX_EVENT_CAP`). Ceiling protects memory (~250 MB per 1M-event run held in RAM) and the viewer.
2. "Win" still means FIRST Jungle King kill; `won`, `deaths`, `final_level`, `level_hist`, `wins`, `avg_deaths`, `avg_final_level` keep their PRE-WIN meaning so win-rate stats are unchanged. After the win the hero keeps playing until the event cap. A run that LOSES (boss_attempts deaths to the boss) still ends there (no post-game); a run that hits the cap before winning ends capped with won=false.
3. Post-game: hero is not sent to the boss again (zone 8 never targeted once won); hunts zones by level (best zone 7); hero level cap rises from 40 to knob `postgame_level_cap` (default 80, range 40..=200; 40 = no post-game levelling), xp_to_next formula continues. Deaths respawn at camp as today. Post-game ends only at the event cap (no other end condition; the flee/death events consume cap so loops cannot spin).
4. Evolution trigger: world stage = min(4, post_kills / evolve_kills) where post_kills = non-boss monster victories after the win; knob `evolve_kills` default 300, range 10..=100000. Stage is 0 before the win, so pre-win history is byte-identical. When the world stage increases emit an event kind `"evolve"` (mon None) before the next encounter. Each encounter's monster carries `stage` (MonSnap.stage: u8, 0..=4). Stage multiplies monster hp, atk, def (applied in spawn only when stage>0, preserving float order at stage 0) by `data::STAGE_MULT = [1.0, 1.3, 1.7, 2.2, 3.0]`. Behaviour, zone and emoji stay per species (stage badge shown by the viewer); the 8 behaviours are unchanged across a line. The boss is never evolved (stage 0).
5. Stage names: `data::EVOLUTIONS: [[&str;5];16]`, `[i][0] == SPECIES[i].name` exactly (so stage-0 text is unchanged); placeholder names for the rest (e.g. Jungle Rat, Dire Rat, Plague Rat, Rat Lord, Rat King); boss row all "Jungle King". `data::stage_name(species, stage) -> &'static str`. Encounter text uses stage_name.
6. Pre-win history unchanged IS achievable: only emit() cap changes; evolution/level cap/zone changes apply only when `won`. Prefix property: events of a run with cap C1 are a prefix of the run with cap C2>C1 (same seed/config).

### Baselines that CHANGE (stated explicitly)
- `simulate(seed).events.len()` for winning seeds: was 10625/10809/10617 (seeds 1,7,42), now 65536 (cap). `first_win_event` = Some(10624/10808/10616); events[..=first_win_event] identical to the old run.
- Old `run_json` bytes change (new fields; stage only emitted when >0, so pre-win events keep their old JSON bytes). Old run.js is still loadable by the viewer.
- Existing tests to update: sim/tests/config_tests.rs `default_matches_baseline` (use first_win_event+1 for the old lengths), `knobs_cover_all_fields` count 13->16 and new defaults; sim/tests/sim_tests.rs `KINDS` gets "evolve", `terminates_under_cap` uses 65536; sim/tests/report_tests.rs `json_escapes_quotes` Run literal gains the new fields.
- UNCHANGED: win rate batch(2000)=1076 wins (53.8%), batch(200)=108, avg_deaths, avg_final_level, level_hist.

## API contract
data.rs: `pub const MAX_STAGE: usize = 4; pub static EVOLUTIONS: [[&str;5];16]; pub static STAGE_MULT: [f32;5]; pub fn stage_name(species:usize, stage:usize)->&'static str`.
config.rs: Config gains `event_cap:u32 (65536)`, `postgame_level_cap:u32 (80)`, `evolve_kills:u32 (300)`; 3 new Knobs (16 total) with set/validate; `pub const MAX_EVENT_CAP: u32 = 1_048_576`.
sim.rs: `MonSnap.stage: u8`. `Run` gains `first_win_event: Option<u32>`, `end_level: u32`, `post_kills: u32`, `post_deaths: u32`, `max_stage: u8`, `capped: bool` (stopped by event cap). `won/deaths/final_level` pre-win semantics (deaths = deaths before first win; all deaths if no win; final_level = level when run ended pre-win, i.e. at the win). New `pub struct Summary { won, deaths, final_level, first_win_event: Option<u32>, events: u32, end_level, post_kills, post_deaths, max_stage: u8, capped }` (derive Clone, Debug, PartialEq), `Run::summary(&self)->Summary`, and `pub fn simulate_summary(seed, &Config)->Summary` that runs in LEAN mode (no event/text/snapshot allocation, same RNG consumption) and must equal `simulate_with(seed,cfg).summary()`. Removes the old const EVENT_CAP in favour of cfg.event_cap.
report.rs: run_body adds top-level `"first_win_event"` (number or null), `"end_level"`, `"post_kills"`, `"post_deaths"`, `"max_stage"`, `"capped"`, `"total"` (events.len()) and per-event `"stage":N` inside "mon" ONLY when N>0. New `pub fn run_window_body(run:&Run, from:usize, count:usize)->String`: same top-level fields plus `"from"`, `"events"` = slice [from, from+count) clamped, and `"level_series"` = hero level sampled evenly, at most 2048 points over ALL events. BatchStats gains `avg_events:f64`, `avg_first_win_event:f64` (mean over wins, 0 if none), `avg_post_kills:f64` and `avg_post_deaths:f64` (mean over wins), `stage_hist:Vec<u32>` (len 5, max stage reached counted over winning runs, sum == wins), stats_body emits all. batch_with uses simulate_summary (lean). Thread independence retained.
server.rs: `/api/config` adds `"max_events":1048576`. event_cap validated via the knob range (400 above). `/api/run`: optional `from`,`count` (u64; count 1..=5000, else 400) -> run_window_body. Without from/count the full run_body is returned only if it has <= 200_000 events, else 400 `{"error":"... use from and count"}`. `/api/batch`: 400 if runs * event_cap > 200_000_000 (2000 x 65536 = 131M passes). Existing caps unchanged.
Performance budget: `cargo run --release -- batch --runs 2000` at the default cap <= 6 s wall on the dev machine (test asserts < 15 s as a CI-safe bound); single `/api/run` window request for a 65536-event run < 1 s.
Viewer: when `run.total > run.events.length` the viewer is in windowed mode and fetches `/api/run?seed=..&<knobs>&from=A&count=2000` (stub-fetchable) when the cursor leaves the loaded window; timeline uses `run.level_series` when present. When a monster has `stage>0` the battle panel shows a `#stage` element containing text `Stage N` (N 1..4) and the stage name; dashboard shows `#post-stats` containing avg events, avg first-win event, avg post kills, avg post deaths and stage_hist counts. file:// with old run.js/stats.js (no new fields) must still work (fields optional). `window.__jv` keeps setRun/goto/logIndices.

## Workstreams
Dependencies: A -> B -> C -> D.

### A core-postgame (sonnet)
Task: data.rs (EVOLUTIONS, STAGE_MULT, stage_name), config.rs (3 knobs), sim.rs (event_cap from cfg, post-game continuation, level cap, stages, evolve event, Run fields, Summary, lean simulate_summary). Commits (one file each): 1) sim/src/data.rs 2) sim/src/config.rs 3) sim/src/sim.rs. Update the existing tests listed above in separate commits (config_tests.rs, sim_tests.rs, report_tests.rs). 
Tests: sim/tests/postgame_tests.rs (config, prefix, cap, post-game, evolution, determinism, summary parts), sim/tests/config_tests.rs, sim/tests/sim_tests.rs.
Acceptance: all A tests pass; pre-win prefix identical; batch(200).wins==108.
Verify: `cargo test --release`.

### B report-stats (sonnet)
Task: run_body fields, run_window_body, lean batch_with, BatchStats fields, stats_body. Commit: sim/src/report.rs. Tests: postgame_tests.rs (report/stats parts), report_tests.rs, report_cfg_tests.rs. Acceptance: old fields unchanged, thread independent, batch(2000) < 15 s test and ~6 s manual: `cargo run --release -- batch --runs 2000`.

### C server-api (sonnet)
Task: server.rs validation/caps/window endpoint/config max_events; README.md section (new knobs, caps, window params); main.rs `run` unchanged (writes full run.js; add `--cap N` optional). Commits: server.rs, main.rs (if touched), README.md. Tests: sim/tests/server_tests.rs + postgame_server_tests.rs. Acceptance: caps as contract; curl check `curl "127.0.0.1:8099/api/run?seed=1&from=0&count=50"`.

### D viewer-postgame (sonnet)
Task: viewer/index.html: windowed fetch, level_series timeline, stage badge `#stage`, `#post-stats` dashboard block, event_cap/postgame knobs auto-render (knobs come from /api/config; verify they appear), evolve events styled in log. Commits: 1) JS data handling 2) HTML/CSS stage badge and post-stats (same file twice allowed). Also regenerate viewer/data/run.js and stats.js with the new build in a final separate commit only if the user wants (open question). Tests: tests/viewer_postgame_check.mjs plus existing tests/*.mjs green. Acceptance: viewer tests pass; 65536-event stub run scrubs and logs fast (goto across run < 200 ms in node stub).

## Test cases
Rust, sim/tests/postgame_tests.rs (A and B):
- config: defaults (event_cap 65536, postgame_level_cap 80, evolve_kills 300); 16 knobs with ranges event_cap 100..=1048576, postgame_level_cap 40..=200, evolve_kills 10..=100000; set accepts bounds, rejects 99, 1048577, 39, 201, 9, 100001, "abc", NaN.
- prefix: seeds 1,7,42 first_win_event == Some(10624/10808/10616), won, final_level 40; events with cap 5000 are a text/kind prefix of default run; events[..=first_win] have no mon.stage>0; batch(200).wins == 108, batch(2000).wins == 1076.
- cap: default winning run has len 65536 and capped; cap 2000 -> len 2000, idx consecutive, capped; len <= cap for caps 100/777/5000 over seeds 0..10; losing seed (found in 0..200, !won) ends with len < 65536, capped false, post_kills 0, no post-game.
- post-game: first win exactly one "win" event at first_win_event; events continue after; no "boss" kind after the win; end_level > 40 and <= postgame_level_cap; postgame_level_cap=40 -> end_level 40; hp invariants hold across post-game; deaths field equals pre-win death count (count "death" events up to first_win).
- evolution: tables (STAGE_MULT strictly increasing, [0]==1.0; EVOLUTIONS[i][0]==SPECIES[i].name; stage names within a non-boss line distinct and nonempty); evolve_kills=10 -> max_stage 4, exactly 4 "evolve" events, stages non-decreasing across encounters post-win; evolve_kills=100000 -> max_stage 0, no evolve events; stage>0 encounter text contains stage_name; stage>0 monster max_hp > stage-0 max_hp of the same species at same level on average.
- determinism: same seed+cfg identical (texts, stages, summary); different evolve_kills differ; simulate_summary == simulate_with(..).summary() for seeds 0..20 incl. non-default cfg.
- stats/report: stage_hist len 5 sums to wins; avg_events > 10000 at default; thread independence (1 vs 4 threads) for non-default cfg; stats_body has new keys; run_body has new top-level keys, `"stage"` absent in pre-win events; run_window_body slice length, from clamp, level_series <= 2048; timing batch(2000) < 15 s.
Rust server (sim/tests/postgame_server_tests.rs): config has event_cap knob and max_events; event_cap=1048577 400; event_cap=2000 200 and body events length 2000; full run with cap 300000 and no window 400; from/count window 200 with `"from"`; count=5001 400, count=0 400; batch runs=2000&event_cap=200000 400; default batch runs=50 200.
Node (tests/viewer_postgame_check.mjs): stage badge shows `Stage 3` for mon.stage 3 and hidden at 0; old-format run (no total/level_series/stage) still renders; windowed mode fetches `from=`/`count=` when goto leaves the window; `#post-stats` shows stage_hist and averages; 65536-event run goto sweep fast.

## Acceptance criteria
- `cargo test --release` and `node tests/*.mjs` green; commits one file each on feat/jungle-sim.
- Win rate/wins/deaths/level_hist baselines unchanged; documented baseline changes only.
- Default run reaches 65536 events; custom cap works up to 1,048,576, rejected above.
- 2000-run default batch <= 6 s manual.
