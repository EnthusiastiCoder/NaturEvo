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
