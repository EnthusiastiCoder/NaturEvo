// Assumed modules:
//   src/hud.ts: partHealthColor(hp): string (hex, 4 tiers), hudModel(frame), defenceTallyModel(defence)
//   src/renderer.ts: rendererOptions(dpr)
//   src/robot-draw.ts must exist and be vector-only
import { describe, it, expect } from 'vitest';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { partHealthColor, hudModel, defenceTallyModel } from '../src/hud';
import { rendererOptions } from '../src/renderer';
import { parseFight } from '../src/data';

const fight = () => parseFight(JSON.parse(readFileSync(resolve(__dirname, 'fixtures/fight_small.json'), 'utf8')));

describe('partHealthColor', () => {
  it('has four distinct tiers: >=75, 50-74, 1-49, 0', () => {
    const tiers = [partHealthColor(100), partHealthColor(75), partHealthColor(74), partHealthColor(50),
      partHealthColor(49), partHealthColor(1), partHealthColor(0)];
    expect(tiers[0]).toBe(tiers[1]);
    expect(tiers[2]).toBe(tiers[3]);
    expect(tiers[4]).toBe(tiers[5]);
    expect(new Set([tiers[0], tiers[2], tiers[4], tiers[6]]).size).toBe(4);
  });
});

describe('hudModel', () => {
  it('has per-fighter bars and a 13-part silhouette', () => {
    const m: any = hudModel(fight().frames[0]);
    expect(m.fighters).toHaveLength(2);
    for (const f of m.fighters) {
      expect(f.parts).toHaveLength(13);
      expect(typeof f.energy).toBe('number');
      expect(typeof f.posture).toBe('number');
      expect(typeof f.coreHp).toBe('number');
      for (const p of f.parts) expect(typeof p.color).toBe('string');
    }
  });
  it('silhouette colour follows part hp', () => {
    const fr: any = JSON.parse(JSON.stringify(fight().frames[0]));
    fr.f[0].hp[6] = 0;
    const m: any = hudModel(fr);
    expect(m.fighters[0].parts[6].color).toBe(partHealthColor(0));
    expect(m.fighters[0].parts[0].color).toBe(partHealthColor(100));
  });
});

describe('defenceTallyModel', () => {
  it('shows parry/block/dodge counts with tooltips', () => {
    const m: any = defenceTallyModel({ parry: 3, block: 2, dodge: 1, hit: 5 });
    const byKey = (k: string) => m.items.find((i: any) => i.key === k);
    expect(byKey('parry').count).toBe(3);
    expect(byKey('block').count).toBe(2);
    expect(byKey('dodge').count).toBe(1);
    for (const k of ['parry', 'block', 'dodge']) {
      expect(typeof byKey(k).tooltip).toBe('string');
      expect(byKey(k).tooltip.length).toBeGreaterThan(5);
    }
  });
});
