# WS5 viewer tests index

Run: `cd v2/viewer && npx vitest run`. Currently 11 failed / 56 passed (fx.test.ts fails to import `../src/fx`).

## Files (new cases)
- tests/fx.test.ts (new, ~20): status table, timer, tint priority, knockback, shake, stars, dotEvents, statusStats, attackName, eyeAlpha.
- tests/parse-fight-fx.test.ts (new, 7): parseFight old and new logs, demoFight coverage.
- tests/interpolation.test.ts (+1): sampleFight carries atk, fx, seen, ev from the earlier frame.
- tests/camera.test.ts (+1): stateLabel for GuardRaise, GuardRecover, Staggered, Stunned, Knocked, Recovery.
- tests/visual-fixes.test.ts (+1): hudOffsets status row ordering and linear scaling.
- tests/ui.test.ts (+1): style.css has `--fx-{knock,stagger,stun,confused,slowed,bleed,poison}`.

## API assumed (implementer must match)
- data.ts: FighterFrame `atk` (-1), `fx` (number[7], zeros), `seen` (false); Frame `ev: []`; Fight `species`, `rank` (passed through as given, null if missing), `monsterName` (from `monster_name`, null). `species`/`rank` are arrays `[hero, monster]`.
- fx.ts: `STATUSES: {key,label,color:number,info}[]` order knock,stagger,stun,confused,slowed,bleed,poison; `statusTimer(key,remaining,hz)` = `"stun 0.8"`; `tintStatus(fx:number[])` returns key or null (stun>stagger>confused>slowed>poison>bleed, knock ignored); `KNOCK_TICKS=6`; `knockbackOffset(remaining,total,dist,dir:[x,y]) -> [x,y]` = -dir*dist*remaining/total (exact at endpoints, monotone, `[0,0]` if total 0); `staggerShake(t,remaining)` 0 when remaining<=0, |v|<=1, deterministic; `stunStarCount(remaining)` 0 or 1..5; `dotEvents(ev)` filters `t==='dot'`; `statusStats(frames,upTo) -> {hero,monster}: {statuses: Record<key,number> (all 7 keys, in order), punishes, parries}` counting hit.fx indexes by `hit.by`, `punish`, and `parry.by`, frames 0..upTo inclusive; `attackName(species,kind,rank)` species strings hero/minion/rogue/warrior/mage, '' for kind -1; `eyeAlpha(seen)` 1 / in (0,0.6).
- layout.ts: stateLabel texts GUARD UP, GUARD DOWN, STAGGER, STUN, KNOCKED, RECOVER (all distinct); `hudOffsets(u)` adds `statusRowTop`, `statusRowBottom` with labelBottom <= top < bottom < barTop.
- demoFight: all 7 fx indexes nonzero somewhere; a hit event with punish true.

## Notes
- Event shapes follow the plan log contract. Rust tests untouched. Nothing committed.
