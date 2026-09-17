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

export const statusTimer = (key: string, remaining: number, hz: number): string => `${key} ${(remaining / hz).toFixed(1)}`;

const TINT_ORDER = ['stun', 'stagger', 'confused', 'slowed', 'poison', 'bleed'];
export function tintStatus(fx: number[]): string | null {
  for (const k of TINT_ORDER) if ((fx[KEYS.indexOf(k)] ?? 0) > 0) return k;
  return null;
}
export const statusColor = (key: string): number => STATUSES.find((s) => s.key === key)?.color ?? 0xffffff;

/** Sprite lags behind the sim's instant jump and catches up: -dir * dist * remaining/total. */
export function knockbackOffset(remaining: number, total: number, dist: number, dir: [number, number]): [number, number] {
  if (total <= 0) return [0, 0];
  const k = Math.max(0, Math.min(1, remaining / total)) * dist;
  return [-dir[0] * k || 0, -dir[1] * k || 0];
}
export const staggerShake = (t: number, remaining: number): number => (remaining <= 0 ? 0 : Math.sin(t * 90) * Math.cos(t * 37));
export const stunStarCount = (remaining: number): number => (remaining <= 0 ? 0 : Math.max(1, Math.min(5, Math.ceil(remaining / 4))));

export interface FightEvent { t: string; [k: string]: unknown }
export const dotEvents = (ev: FightEvent[]): FightEvent[] => ev.filter((e) => e.t === 'dot');
