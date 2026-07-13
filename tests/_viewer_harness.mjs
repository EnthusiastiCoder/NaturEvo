// Shared helper for viewer node tests: loads the inline script of viewer/index.html
// in node:vm with a small stub DOM, stub fetch and stub timers. No dependencies.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import vm from 'node:vm';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
export const htmlPath = join(root, 'viewer', 'index.html');
export const html = readFileSync(htmlPath, 'utf8');

export function makeRun(n, seed = 1) {
  const events = [];
  for (let i = 0; i < n; i++) {
    events.push({
      idx: i, kind: 'travel', text: 'event ' + i,
      hero: { level: 1 + Math.floor(i / 20), hp: 10, max_hp: 10, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0 },
      mon: null,
    });
  }
  return { seed, events, won: false, deaths: 0, final_level: 1 };
}
