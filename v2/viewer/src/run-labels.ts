// Pure helpers that make the run list readable: humane names, scratch detection, grouping, default pick.
import type { RunInfo } from './runs-index';

export const hasCampaign = (r: RunInfo): boolean => !!(r.permadeath.on || r.permadeath.off);
