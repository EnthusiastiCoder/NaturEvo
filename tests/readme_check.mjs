// README requirements for the hero map/walking feature (plans/2026-10-21_hero-map-walking.md, workstream 4).
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { runner } from './_viewer_harness.mjs';

const { check, done } = runner('readme_check');
const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const readme = readFileSync(join(root, 'README.md'), 'utf8');
const cfg = readFileSync(join(root, 'sim', 'src', 'config.rs'), 'utf8');
