import type { Frame, Tally } from './data';

/** Four tiers: >=75 healthy, 50-74 worn, 1-49 critical, 0 disabled. */
export function partHealthColor(hp: number): string {
  if (hp >= 75) return '#3fb950';
  if (hp >= 50) return '#d4b41c';
  if (hp >= 1) return '#e0701a';
  return '#7a2f2f';
}

export function hudModel(frame: Frame) {
  return {
    fighters: frame.f.map((g) => ({
      parts: g.hp.map((hp) => ({ color: partHealthColor(hp) })),
      energy: g.energy, posture: g.posture, coreHp: g.hp[1] ?? 0,
    })),
  };
}

export function defenceTallyModel(d: Tally) {
  return {
    items: [
      { key: 'parry', count: d.parry, tooltip: 'Parries: incoming blade deflected with the sword' },
      { key: 'block', count: d.block, tooltip: 'Blocks: incoming blow absorbed by the shield' },
      { key: 'dodge', count: d.dodge, tooltip: 'Dodges: attack evaded by moving or ducking' },
    ],
  };
}
