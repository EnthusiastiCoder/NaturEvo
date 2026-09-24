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
