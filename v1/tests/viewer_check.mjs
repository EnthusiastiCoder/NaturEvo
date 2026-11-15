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

for (const id of ['tab-replay', 'tab-dashboard', 'play', 'speed', 'scrub', 'log', 'battle', 'timeline', 'hist']) {
  check(new RegExp(`id\\s*=\\s*["']${id}["']`).test(html), `missing element id ${id}`);
}
check(/<script[^>]+src\s*=\s*["']data\/run\.js["']/.test(html), 'missing script tag for data/run.js');
check(/<script[^>]+src\s*=\s*["']data\/stats\.js["']/.test(html), 'missing script tag for data/stats.js');

const inline = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map(m => m[1]);
check(inline.length > 0, 'no inline script');
inline.forEach((code, i) => {
  try { new Function(code); } catch (e) { check(false, `inline script ${i} syntax error: ${e.message}`); }
});

if (failed) { console.error(`${failed} check(s) failed`); process.exit(1); }
console.log('viewer_check OK');
