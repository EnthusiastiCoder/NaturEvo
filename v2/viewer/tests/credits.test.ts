import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';

const credits = resolve(__dirname, '../../assets/CREDITS.md');

describe('CREDITS.md licence check', () => {
  it('exists', () => expect(existsSync(credits)).toBe(true));
  it('declares KayKit CC0 and pins a commit', () => {
    const t = readFileSync(credits, 'utf8');
    expect(t).toMatch(/KayKit/i);
    expect(t).toMatch(/CC0/);
    expect(t).toMatch(/\b[0-9a-f]{7,40}\b/);
  });
  it('lists every committed atlas', () => {
    const t = readFileSync(credits, 'utf8');
    for (const n of ['Skeleton_Warrior', 'Skeleton_Minion', 'Skeleton_Rogue', 'Skeleton_Mage']) {
      expect(t).toContain(n);
    }
  });
  it('no atlas file in assets lacks a CREDITS mention', () => {
    const t = readFileSync(credits, 'utf8');
    const dir = resolve(__dirname, '../../assets');
    const files = readdirSync(dir).filter((f) => /\.(png|json)$/i.test(f));
    for (const f of files) {
      const base = f.replace(/\.(png|json)$/i, '');
      expect(t, `${f} not credited`).toContain(base);
    }
  });
});
