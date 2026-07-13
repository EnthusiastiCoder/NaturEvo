import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const file = join(root, 'viewer', 'index.html');
let html;
try {
  html = readFileSync(file, 'utf8');
} catch {
  console.error('FAIL: cannot read ' + file);
  process.exit(1);
}

let failed = 0;
const check = (ok, msg) => {
  if (!ok) { failed++; console.error('FAIL: ' + msg); }
};
