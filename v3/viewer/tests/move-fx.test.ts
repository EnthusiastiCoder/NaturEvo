// WS-E move FX tests (plan 2026-10-23_v3-sword-vocabulary.md, Viewer section and Test cases WS-E).
//
// INVENTED API, new module src/move-fx.ts (pure functions, no Pixi/DOM). Times are log ticks; `dt` is seconds per tick (fight.dt).
//   interface Callout { text: string; x: number; y: number; alpha: number; color: string; kind: 'move'|'punish'|'counter'|'guard_break' }
//   moveCallout(events: FightEvent[], frame: Frame, tNow: number, dt: number): Callout | null
//     Most recent move/punish/counter/guard_break event with event.t <= tNow < event.t + 0.6/dt, anchored above the head of
//     fighter `who` taken from frame.f[who].parts[0] (Head; x = head x, y above the head). alpha = 1 at the event tick,
//     linear fade to 0 at 0.6 s. Returns null before the event and after 0.6 s (or alpha 0).
//     Text: move -> the move name (e.g. "Overhead"); punish -> contains "PUNISH"; counter -> contains "COUNTER";
//     guard_break -> contains "GUARD BREAK". Colours of the four kinds are all different.
//   interface PhaseStyle { glow: number; trail: boolean; dim: boolean; color: string }
//   phaseStyle(phase: 0|1|2|3, p: number): PhaseStyle
//     0 idle: no glow, no trail, not dim. 1 windup: amber glow in (0,1], growing with p, no trail. 2 active: bright white edge, trail true.
//     3 recovery: dim true, no trail. Colours of windup, active and recovery differ.
//   interface OpenBadge { show: boolean; x: number; y: number; arc: number; riposte: boolean }
//   openBadge(f: FighterFrame): OpenBadge
//     show when f.mv.open is true or f.state === 'staggered'; arc is the remaining fraction 0..1 (open recovery: 1 - mv.p);
//     riposte mirrors mv.rip (the defender's riposte window icon). Frames without mv never throw.
//   hudMoveLine(f: FighterFrame): string  -> "<Name> · <Windup|Active|Recovery>" while moving, "Idle" when idle, "" without mv.
//   interface GraceRow { key: 'whiff'|'punish'|'hits_per_swing'|'jerk'; label: string; value: string; tooltip: string }
//   graceModel(result: Fight['result'], who: 0|1): { rows: GraceRow[]; usage: { name: string; count: number }[] } | null
//     null when result.moves is absent. rows in the order whiff, punish, hits_per_swing, jerk; whiff and punish as whole
//     percent strings ("50%"), hits_per_swing and jerk with 2 decimals; every tooltip a distinct non-empty string.
//     usage sorted by count descending, ties by name.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseFight } from '../src/data';
import { moveCallout, phaseStyle, openBadge, hudMoveLine, graceModel } from '../src/move-fx';

const load = (n: string) => parseFight(JSON.parse(readFileSync(resolve(__dirname, 'fixtures', n), 'utf8')));
const fight = () => load('fight_moves.json');
const frameAt = (f: ReturnType<typeof fight>, t: number) => f.frames.find((fr) => fr.t === t)!;

describe('fixture and parseFight', () => {
  it('parses the move fixture with mv on every fighter frame and all new events', () => {
    const f = fight();
    expect(f.frames).toHaveLength(20);
    for (const fr of f.frames) for (const g of fr.f) {
      const mv = (g as any).mv;
      expect(mv).toBeTruthy();
      expect(typeof mv.id).toBe('string');
      expect([0, 1, 2, 3]).toContain(mv.phase);
      expect(mv.p).toBeGreaterThanOrEqual(0);
      expect(mv.p).toBeLessThanOrEqual(1);
      expect(typeof mv.open).toBe('boolean');
    }
    const kinds = new Set(f.events.map((e) => e.kind));
    for (const k of ['move', 'whiff', 'punish', 'guard_break', 'feint', 'counter', 'riposte_open']) expect(kinds.has(k)).toBe(true);
    expect((f.result as any).moves.usage).toHaveLength(2);
  });
  it('tolerates a log without mv (old fight_small.json)', () => {
    const f = load('fight_small.json');
    expect(f.frames).toHaveLength(12);
    expect((f.frames[0].f[0] as any).mv).toBeUndefined();
  });
});

describe('moveCallout', () => {
  const f = fight();
  const at = (t: number) => moveCallout(f.events, frameAt(f, t - (t % 2)), t, f.dt);
  it('is null before the event', () => { expect(at(2)).toBeNull(); });
  it('names the move near the attacker head, fully opaque at the event tick', () => {
    const c = at(4)!;
    expect(c).not.toBeNull();
    expect(c.text.toLowerCase()).toContain('overhead');
    expect(c.alpha).toBeGreaterThan(0.95);
    const head = frameAt(f, 4).f[0].parts[0];
    expect(Math.abs(c.x - (head[0] + head[2]) / 2)).toBeLessThan(80);
    expect(Math.abs(c.y - Math.max(head[1], head[3]))).toBeLessThan(200);
  });
  it('fades to nothing over 0.6 s', () => {
    const ticks = 0.6 / f.dt; // 18
    const mid = at(4 + Math.round(ticks / 2))!;
    expect(mid.alpha).toBeGreaterThan(0.3);
    expect(mid.alpha).toBeLessThan(0.7);
    const end = at(4 + Math.ceil(ticks) + 1);
    expect(end === null || end.alpha === 0).toBe(true);
    const later = at(10);
    expect(later === null || later.kind !== 'move' || later.alpha < 0.4).toBe(true);
  });
  it('PUNISH, COUNTER and GUARD BREAK variants are distinct', () => {
    const p = at(14)!, c = at(30)!, g = at(20)!, m = at(4)!;
    expect(p.text).toContain('PUNISH');
    expect(c.text).toContain('COUNTER');
    expect(g.text).toContain('GUARD BREAK');
    expect(new Set([p.color, c.color, g.color, m.color]).size).toBe(4);
    expect(new Set([p.kind, c.kind, g.kind, m.kind]).size).toBe(4);
  });
  it('punish anchors on the attacker (who) not the victim', () => {
    const p = at(14)!;
    const att = frameAt(f, 14).f[1].parts[0], vic = frameAt(f, 14).f[0].parts[0];
    const hx = (h: number[]) => (h[0] + h[2]) / 2;
    expect(Math.abs(p.x - hx(att))).toBeLessThan(Math.abs(p.x - hx(vic)));
  });
});

describe('phaseStyle', () => {
  it('idle has no effects', () => {
    const s = phaseStyle(0, 0);
    expect(s.glow).toBe(0);
    expect(s.trail).toBe(false);
    expect(s.dim).toBe(false);
  });
  it('windup glow grows with progress and has no trail', () => {
    const a = phaseStyle(1, 0.2), b = phaseStyle(1, 0.9);
    expect(a.glow).toBeGreaterThan(0);
    expect(b.glow).toBeGreaterThan(a.glow);
    expect(b.glow).toBeLessThanOrEqual(1);
    expect(a.trail).toBe(false);
  });
  it('active has a trail and a bright edge', () => {
    const s = phaseStyle(2, 0.5);
    expect(s.trail).toBe(true);
    expect(s.dim).toBe(false);
    expect(s.color.toLowerCase()).toMatch(/^#f{3,6}$/);
  });
  it('recovery is dim with no trail, and the three phases use three colours', () => {
    const r = phaseStyle(3, 0.5);
    expect(r.dim).toBe(true);
    expect(r.trail).toBe(false);
    expect(new Set([phaseStyle(1, 0.5).color, phaseStyle(2, 0.5).color, r.color]).size).toBe(3);
  });
});

describe('openBadge', () => {
  const f = fight();
  it('is hidden for an idle fighter', () => { expect(openBadge(frameAt(f, 0).f[0]).show).toBe(false); });
  it('shows during an open recovery with the remaining fraction 1 - p', () => {
    const b = openBadge(frameAt(f, 12).f[0]); // Overhead recovery p 0.25, open
    expect(b.show).toBe(true);
    expect(b.arc).toBeGreaterThanOrEqual(0);
    expect(b.arc).toBeLessThanOrEqual(1);
    expect(b.arc).toBeCloseTo(0.75, 2);
  });
  it('shows for the staggered state in frame 9 of fighter 1', () => {
    const fr = f.frames[9];
    expect(fr.f[1].state).toBe('staggered');
    const b = openBadge(fr.f[1]);
    expect(b.show).toBe(true);
    expect(b.arc).toBeGreaterThanOrEqual(0);
    expect(b.arc).toBeLessThanOrEqual(1);
  });
  it('flags the riposte window on the defender', () => {
    expect(openBadge(frameAt(f, 34).f[0]).riposte).toBe(true);
    expect(openBadge(frameAt(f, 0).f[0]).riposte).toBe(false);
  });
  it('does not throw on frames without mv', () => {
    const old = load('fight_small.json');
    expect(() => openBadge(old.frames[0].f[0])).not.toThrow();
    expect(openBadge(old.frames[0].f[0]).show).toBe(false);
  });
});

describe('hudMoveLine', () => {
  const f = fight();
  it('shows the move name and phase', () => {
    expect(hudMoveLine(frameAt(f, 12).f[0])).toBe('Overhead · Recovery');
    expect(hudMoveLine(frameAt(f, 4).f[0])).toBe('Overhead · Windup');
    expect(hudMoveLine(frameAt(f, 20).f[0])).toBe('ShieldBash · Active');
  });
  it('says Idle when idle and is empty for old logs', () => {
    expect(hudMoveLine(frameAt(f, 0).f[0])).toBe('Idle');
    expect(hudMoveLine(load('fight_small.json').frames[0].f[0])).toBe('');
  });
});

describe('graceModel', () => {
  const f = fight();
  it('lists whiff, punish, hits per swing and jerk with formatted values and tooltips', () => {
    const g = graceModel(f.result, 0)!;
    expect(g.rows.map((r) => r.key)).toEqual(['whiff', 'punish', 'hits_per_swing', 'jerk']);
    expect(g.rows.map((r) => r.value)).toEqual(['50%', '0%', '0.80', '0.12']);
    for (const r of g.rows) { expect(r.label.length).toBeGreaterThan(0); expect(r.tooltip.length).toBeGreaterThan(5); }
    expect(new Set(g.rows.map((r) => r.tooltip)).size).toBe(4);
  });
  it('is per fighter', () => {
    const g = graceModel(f.result, 1)!;
    expect(g.rows.map((r) => r.value)).toEqual(['25%', '40%', '0.60', '0.31']);
  });
  it('sorts move usage by count descending', () => {
    const g = graceModel(f.result, 0)!;
    expect(g.usage[0]).toEqual({ name: 'Overhead', count: 6 });
    expect(g.usage.map((u) => u.name)).toEqual(['Overhead', 'Thrust', 'Parry', 'ShieldBash']);
  });
  it('is null when the log has no move summary', () => {
    expect(graceModel(load('fight_small.json').result, 0)).toBeNull();
  });
});
