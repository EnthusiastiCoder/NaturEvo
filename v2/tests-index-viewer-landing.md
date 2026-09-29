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
