// README requirements for the hero map/walking feature (plans/2026-10-21_hero-map-walking.md, workstream 4).
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { runner } from './_viewer_harness.mjs';

const { check, done } = runner('readme_check');
const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const readme = readFileSync(join(root, 'README.md'), 'utf8');
const cfg = readFileSync(join(root, 'sim', 'src', 'config.rs'), 'utf8');

// every knob key in KNOBS is documented (this includes the 6 new ones)
const keys = [...cfg.matchAll(/Knob\s*\{\s*key:\s*"([a-z_]+)"/g)].map(m => m[1]);
check(keys.length > 0, 'could not read KNOBS from config.rs');
for (const k of ['map_size', 'encounter_rate', 'wander_radius', 'elites_per_zone', 'elite_mult', 'max_steps']) {
  check(keys.includes(k), 'KNOBS must contain ' + k);
}
for (const k of keys) check(readme.includes(k), 'README must mention knob ' + k);

check(readme.includes('/api/map'), 'README must document the /api/map route');
check(/^#{1,4}\s.*\bmove tables?\b/im.test(readme), 'README needs a "Move tables" heading');
check(/^#{1,4}\s.*\b(hero (skills|moveset)|moveset)\b/im.test(readme), 'README needs a hero skills/moveset heading');
for (const s of ['Power Strike', 'Brace', 'Stun', 'Heal']) check(readme.includes(s), 'README must list the hero skill ' + s);
// event schema additions and the steps-vs-cap note
for (const w of ['walk', 'npcs', 'path', 'steps']) check(new RegExp('\\b' + w + '\\b').test(readme), 'README must mention ' + w);
check(/steps?[^.\n]*(not|never)[^.\n]*events?|events?[^.\n]*(not|never)[^.\n]*steps?/i.test(readme), 'README must note that steps are not events (event cap)');
check(/jv-mapzoom|zoom/i.test(readme), 'README should mention the map zoom in the viewer');
