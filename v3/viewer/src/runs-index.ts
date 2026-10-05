// Run discovery: pure parsing of what the sim writes into v2/runs, plus the client-side index loader.
export type Row = Record<string, number>;
export type CampaignSummary = { outcome: string; wins: number; losses: number; encounters: number; perZone: Record<string, { wins: number; losses: number }> };
export type RunInfo = {
  name: string; seed: string; date: string; generations: number;
  permadeath: { on?: CampaignSummary; off?: CampaignSummary };
  keyResult: string; files: string[];
};
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

export function summarizeCampaign(jsonText: string): CampaignSummary | null {
  try {
    const j = JSON.parse(jsonText);
    if (!j || typeof j !== 'object') return null;
    const encs: { zone?: number; result?: string }[] = Array.isArray(j.encounters) ? j.encounters : [];
    const perZone: CampaignSummary['perZone'] = {};
    let wins = 0, losses = 0;
    for (const e of encs) {
      const win = e.result === 'HeroWin';
      if (win) wins++; else losses++;
      const z = (perZone[String(e.zone ?? 'n/a')] ??= { wins: 0, losses: 0 });
      if (win) z.wins++; else z.losses++;
    }
    return { outcome: typeof j.outcome === 'string' ? j.outcome : 'n/a', wins, losses, encounters: encs.length, perZone };
  } catch { return null; }
}

export function seedOf(name: string): string {
  const m = /(?:^|\/)s(\d+)$/.exec(name);
  return m ? m[1] : 'n/a';
}

export async function buildIndex(src: FileSource, opts: { mtime(path: string): string | undefined }): Promise<RunIndex> {
  const byDir = new Map<string, string[]>();
  for (const p of src.list()) {
    const i = p.lastIndexOf('/');
    if (i < 0) continue;
    const dir = p.slice(0, i);
    (byDir.get(dir) ?? byDir.set(dir, []).get(dir)!).push(p.slice(i + 1));
  }
  const runs: RunInfo[] = [];
  for (const [dir, names] of byDir) {
    const hasCsv = names.includes('training.csv');
    const camps = names.filter((n) => /^campaign_(on|off)\.json$/.test(n));
    if (!hasCsv && !camps.length) continue;
    try {
      const read = async (n: string) => { try { return await src.read(`${dir}/${n}`); } catch { return ''; } };
      const generations = hasCsv ? parseTrainingCsv(await read('training.csv')).length : 0;
      const permadeath: RunInfo['permadeath'] = {};
      if (camps.includes('campaign_on.json')) permadeath.on = summarizeCampaign(await read('campaign_on.json')) ?? undefined;
      if (camps.includes('campaign_off.json')) permadeath.off = summarizeCampaign(await read('campaign_off.json')) ?? undefined;
      runs.push({
        name: dir, seed: seedOf(dir), date: opts.mtime(`${dir}/${hasCsv ? 'training.csv' : camps[0]}`) ?? 'n/a',
        generations, permadeath, keyResult: permadeath.on?.outcome ?? permadeath.off?.outcome ?? 'n/a', files: names.slice().sort(),
      });
    } catch { /* odd folder: skip */ }
  }
  runs.sort((a, b) => a.name.localeCompare(b.name));
  return { runs };
}
