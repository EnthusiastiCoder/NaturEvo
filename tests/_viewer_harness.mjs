// Shared helper for viewer node tests: loads the inline script of viewer/index.html
// in node:vm with a small stub DOM, stub fetch and stub timers. No dependencies.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import vm from 'node:vm';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
export const htmlPath = join(root, 'viewer', 'index.html');
export const html = readFileSync(htmlPath, 'utf8');
