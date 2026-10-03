import { defineConfig } from 'vitest/config';
import { resolve } from 'node:path';
import { runsPlugin } from './vite-plugin-runs';

export default defineConfig({
  plugins: [runsPlugin()],
  server: { port: 5175, strictPort: true, fs: { allow: [resolve(__dirname, '..')] } },
  build: { target: 'es2022' },
  test: { environment: 'node', include: ['tests/**/*.test.ts'] },
});
