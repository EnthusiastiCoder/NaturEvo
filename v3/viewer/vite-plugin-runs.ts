import type { Plugin } from 'vite';
import { createReadStream, existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, resolve, sep } from 'node:path';
import { buildIndex } from './src/runs-index';
