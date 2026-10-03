// Static check on v3/viewer/src/style.css: no page scroll.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const css = () => readFileSync(resolve(__dirname, '../src/style.css'), 'utf8');

describe('layout', () => {
  it('root clips overflow (no page scroll)', () => {
    expect(css()).toMatch(/overflow\s*:\s*hidden/);
  });
  it('root fills the viewport', () => {
    expect(css()).toMatch(/100vh|height\s*:\s*100%/);
  });
});
