// Run discovery: pure parsing of what the v3 sim writes into v3/runs, plus the client-side index loader.
export type Row = Record<string, number>;
export type RunInfo = { name: string; seed: string; date: string; generations: number; fightGens: number[] };
export type RunIndex = { runs: RunInfo[]; error?: string };
export type FileSource = { list(): string[]; read(path: string): Promise<string> };

export function parseTrainingCsv(text: string): Row[] {
  const lines = text.split(/\r?\n/).filter((l) => l.trim() !== '');
  if (lines.length < 2) return [];
  const cols = lines[0].split(',').map((c) => c.trim());
  return lines.slice(1).map((l) => {
    const cells = l.split(',');
    const row: Row = {};
    cols.forEach((c, i) => { const v = Number(cells[i]); if (cells[i] !== undefined && cells[i] !== '' && Number.isFinite(v)) row[c] = v; });
    return row;
  });
}

/** One run = one folder holding run.json; fight generations come from run.json, else from fight_gen_N.json names. */
export async function buildIndex(src: FileSource, opts: { mtime(path: string): string | undefined }): Promise<RunIndex> {
  const dirs = new Map<string, string[]>();
  for (const p of src.list()) {
    const i = p.lastIndexOf('/');
    if (i < 0) continue;
    const d = p.slice(0, i);
    if (!dirs.has(d)) dirs.set(d, []);
    dirs.get(d)!.push(p.slice(i + 1));
  }
  const runs: RunInfo[] = [];
  for (const [dir, names] of dirs) {
    if (!names.includes('run.json')) continue;
    try {
      const j = JSON.parse(await src.read(`${dir}/run.json`));
      const fromFiles = names.map((n) => /^fight_gen_(\d+)\.json$/.exec(n)).filter((m): m is RegExpExecArray => !!m).map((m) => Number(m[1]));
      const fg: number[] = (Array.isArray(j.fight_gens) ? j.fight_gens : fromFiles).filter((x: unknown) => Number.isFinite(x)).sort((a: number, b: number) => a - b);
      runs.push({
        name: String(j.name ?? dir), seed: j.seed === undefined ? 'n/a' : String(j.seed),
        date: typeof j.date === 'string' ? j.date : opts.mtime(`${dir}/run.json`) ?? 'n/a',
        generations: Number(j.generations) || 0, fightGens: fg,
      });
    } catch { /* odd folder: skip */ }
  }
  runs.sort((a, b) => a.name.localeCompare(b.name));
  return { runs };
}

export async function loadIndex(fetchFn: typeof fetch = fetch): Promise<RunIndex> {
  const reasons: string[] = [];
  for (const url of ['/api/runs', '/runs/index.json']) {
    try {
      const r = await fetchFn(url);
      if (!r.ok) { reasons.push(`${url}: HTTP ${r.status}`); continue; }
      const j = await r.json();
      if (j && Array.isArray(j.runs)) return { runs: j.runs };
      reasons.push(`${url}: unexpected shape`);
    } catch (e) { reasons.push(`${url}: ${e instanceof Error ? e.message : String(e)}`); }
  }
  return { runs: [], error: reasons.join('; ') };
}
