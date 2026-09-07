import { defineConfig } from 'vitest/config';
import { resolve } from 'node:path';

export default defineConfig({
  server: { port: 5174, strictPort: true, fs: { allow: [resolve(__dirname, '..')] } },
  build: { target: 'es2022' },
  test: { environment: 'node', include: ['tests/**/*.test.ts'] },
});
