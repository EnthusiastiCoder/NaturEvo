// Pure helpers for status effects, knockback and attack labels (no rendering).
import type { Frame } from './interp';

export interface StatusDef { key: string; label: string; color: number; info: string }
export const STATUSES: StatusDef[] = [
  { key: 'knock', label: 'Knockback', color: 0xffffff, info: 'Pushed away by the hit; short visual slide.' },
  { key: 'stagger', label: 'Stagger', color: 0xffa94d, info: 'Staggered: cannot act until it wears off.' },
  { key: 'stun', label: 'Stun', color: 0xffd43b, info: 'Stunned: helpless, takes reduced damage.' },
  { key: 'confused', label: 'Confused', color: 0xda77f2, info: 'Confused: aim and turning go wrong.' },
  { key: 'slowed', label: 'Slowed', color: 0x4dabf7, info: 'Slowed: moves at reduced speed.' },
  { key: 'bleed', label: 'Bleed', color: 0xe03131, info: 'Bleeding: loses hp every tick.' },
  { key: 'poison', label: 'Poison', color: 0x82c91e, info: 'Poisoned: loses hp over time.' },
];
export const KEYS = STATUSES.map((s) => s.key);
export const KNOCK_TICKS = 6;
