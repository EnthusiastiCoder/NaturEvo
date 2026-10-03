import type { Plugin } from 'vite';
import { createReadStream, existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, resolve, sep } from 'node:path';
import { buildIndex } from './src/runs-index';

const SKIP = new Set(['node_modules', '.git']);

function walk(root: string, rel = '', depth = 0, out: string[] = []): string[] {
  if (depth > 3 || !existsSync(join(root, rel))) return out;
  for (const e of readdirSync(join(root, rel), { withFileTypes: true })) {
    if (SKIP.has(e.name)) continue;
    const r = rel ? `${rel}/${e.name}` : e.name;
    if (e.isDirectory()) walk(root, r, depth + 1, out); else out.push(r);
  }
  return out;
}
