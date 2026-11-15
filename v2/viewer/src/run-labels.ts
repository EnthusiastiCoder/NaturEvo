// Pure helpers that make the run list readable: humane names, scratch detection, grouping, default pick.
import type { RunInfo } from './runs-index';

export const hasCampaign = (r: RunInfo): boolean => !!(r.permadeath.on || r.permadeath.off);

export function humanName(name: string): string {
  return name.split('/').map((seg) => {
    const seed = /^s(\d+)$/.exec(seg);
    if (seed) return `seed ${seed[1]}`;
    return seg.replace(/_r(\d+)$/, ' round $1').replace(/_/g, ' ');
  }).join(' / ');
}

export const isScratch = (name: string): boolean =>
  name.split('/').some((s) => /^tmp/.test(s) || /^t200s/.test(s) || /^_v/.test(s) || /_tmp$/.test(s));

const time = (r: RunInfo) => { const t = Date.parse(r.date); return Number.isFinite(t) ? t : 0; };
const newest = (a: RunInfo, b: RunInfo) => time(b) - time(a) || a.name.localeCompare(b.name);

export function groupRuns(runs: RunInfo[]): { campaign: RunInfo[]; training: RunInfo[]; scratch: RunInfo[] } {
  const live = runs.filter((r) => !isScratch(r.name));
  return {
    campaign: live.filter(hasCampaign).sort(newest),
    training: live.filter((r) => !hasCampaign(r)).sort(newest),
    scratch: runs.filter((r) => isScratch(r.name)).sort(newest),
  };
}

/** Newest run with campaign data (real runs before scratch), else the newest run. */
export function defaultRun(runs: RunInfo[]): RunInfo | undefined {
  const g = groupRuns(runs);
  return g.campaign[0] ?? runs.filter(hasCampaign).sort(newest)[0] ?? g.training[0] ?? g.scratch[0];
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
export function formatDate(s: string): string {
  const t = Date.parse(s);
  if (!Number.isFinite(t)) return '';
  const d = new Date(t);
  if (!s.includes('T')) return `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]}`;
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getDate()} ${MONTHS[d.getMonth()]} ${p(d.getHours())}:${p(d.getMinutes())}`;
}
