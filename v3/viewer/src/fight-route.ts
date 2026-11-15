// Hash routes that open a fight: #/fight/<run>/<n> (a generation's fight) and #/featured/<featured_name> (a long featured fight).
export type FightRoute = { url: string; title: string };

export function fightRoute(hash: string): FightRoute | null {
  const g = /^#\/fight\/([^/]+)\/(\d+)$/.exec(hash);
  if (g) return { url: `/runs/${g[1]}/fight_gen_${g[2]}.json`, title: `${decodeURIComponent(g[1])} gen ${g[2]}` };
  const f = /^#\/featured\/(featured_[A-Za-z0-9_-]+)$/.exec(hash);
  if (f) return { url: `/runs/${f[1]}.json`, title: f[1].replace(/_/g, ' ') };
  return null;
}
