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

export function runsPlugin(dir = process.env.NV_RUNS_DIR ?? resolve(__dirname, '..', 'runs')): Plugin {
  const index = () => buildIndex(
    { list: () => walk(dir), read: async (p) => readFileSync(join(dir, p), 'utf8') },
    { mtime: (p) => { try { return statSync(join(dir, p)).mtime.toISOString().slice(0, 10); } catch { return undefined; } } },
  );
  return {
    name: 'nv-runs',
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        const url = (req.url ?? '').split('?')[0];
        if (url === '/api/runs') {
          res.setHeader('content-type', 'application/json'); res.setHeader('cache-control', 'no-store');
          res.end(JSON.stringify(await index()));
        } else if (url.startsWith('/runs/')) {
          const file = resolve(dir, decodeURIComponent(url.slice(6)));
          if (!file.startsWith(resolve(dir) + sep) || !existsSync(file) || !statSync(file).isFile()) { res.statusCode = 404; res.end('not found'); return; }
          res.setHeader('content-type', file.endsWith('.json') ? 'application/json' : 'text/plain');
          createReadStream(file).pipe(res);
        } else next();
      });
    },
    async generateBundle() {
      this.emitFile({ type: 'asset', fileName: 'runs/index.json', source: JSON.stringify(await index()) });
    },
  };
}
