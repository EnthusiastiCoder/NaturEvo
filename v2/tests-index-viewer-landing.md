# Tests index: viewer landing (plans/2026-10-23_v2-viewer-landing.md)

Run: `cd v2/viewer && npx vitest run tests/runs-index.test.ts tests/aggregate.test.ts tests/landing.test.ts tests/routing.test.ts`

## v2/viewer/tests/runs-index.test.ts -> src/runs-index.ts
Exports: `parseTrainingCsv(text): Row[]` (numbers keyed by column; header-only/empty -> []),
`summarizeCampaign(jsonText): {outcome,wins,losses,encounters,perZone:{[zone]:{wins,losses}}} | null` (result 'HeroWin' = win, else loss),
`buildIndex(src:{list():string[]; read(path):Promise<string>}, opts:{mtime(path):string|undefined}): Promise<{runs:RunInfo[]}>` (paths relative to runs dir, e.g. `main/training.csv`, `cd/s1/training.csv`; run = folder containing training.csv or campaign_*.json; name "cd/s1" for nested),
`loadIndex(fetchFn): Promise<{runs, error?}>` (/api/runs, then /runs/index.json, else `{runs:[],error:string}`).
- parse csv rows; empty csv
- campaign summary counts and perZone; invalid JSON -> null
- buildIndex: generations=row count, on/off summaries, date from mtime, seed n/a for "main", seed "1" for "cd/s1", files list, nested names
- junk skipped without throw; missing mtime -> "n/a"
- loadIndex: first source, fallback, error state

## v2/viewer/tests/aggregate.test.ts -> src/aggregate.ts
Exports: `campaignWinRate({on?,off?}:{wins,losses}) -> {on,off:{k,n,rate|null}}`, `poolRates([{k,n}]) -> {k,n,rate|null}`,
`learningCurves(rows) -> {gens,heroFitness,monsterFitness,heroWinVsEarlier,monsterWinVsEarlier,drawRate:number[]|null}` (draw = 1 - hof_wr_hero - hof_wr_monster),
`actionShares(ticks:{action,seen,dist}[], {distBuckets:number[]}) -> {byVisibility:{seen,unseen},byDistance:[...]}` each `{n,attack,block,dodge}`; null when fields missing; bucket 0 = dist <= first bound,
`zoneRates(encounters:{zone,result}[]) -> {zone,k,n,rate}[]` sorted by zone,
`statusesPerFight(fights:{frames:{statuses:string[]}[]}[]) -> Record<string,number>[]`.
- win rate on vs off; null rate at n=0; pooling weighted; curve lengths/values; draw rate; empty; action shares by visibility/distance/empty/missing; zone rates; statuses

## v2/viewer/tests/landing.test.ts -> src/landing.ts
Exports: `EMPTY_COMMANDS: string[]` (contains `cargo run --release -- train ...`, `... campaign ...`), `landingViewModel(index) -> {empty, rows:[{name,seed,date,generations,permadeathOn,permadeathOff,keyResult,openHref}]}`, `renderLandingHtml(vm): string` (Demo `<button data-action="demo">`, `data-action="all-runs"` toggle, `<a href="?run=<encodeURIComponent(name)>">`).
- empty state commands, no banner/error/red wording; error index still empty state
- picker row fields; open href; Demo button; no hard-coded colours
