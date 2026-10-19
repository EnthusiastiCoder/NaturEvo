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
