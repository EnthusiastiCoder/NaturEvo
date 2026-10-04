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
