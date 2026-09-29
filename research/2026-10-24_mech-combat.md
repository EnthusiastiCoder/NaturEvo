# v3 research: big-robot, per-part combat (2026-10-24)

Honesty note: sections 1-5 below were first written from background knowledge. A second pass (see "Verified findings" at the end) fetched or searched 9 sources; claims backed there are marked [V]. Everything not marked [V] is still UNVERIFIED background knowledge, and numbers marked (guess) are my design proposals.

## 1. Mech-X4 / giant robot fiction (short)
- Mech-X4 (Disney XD, 2016): teen pilot bonded to a giant humanoid robot; abilities come in as unlockable "powers" (energy blasts, shields, flight-ish boosts). Source: https://en.wikipedia.org/wiki/Mech-X4
- Distinctive: humanoid silhouette (readable head/torso/arms/legs), one signature colour per robot, glowing core/eye, power effects as bright additive glow.
- Takeaway: keep a fixed humanoid part list; give each robot a colour + glow accent; abilities are separate from basic strikes (v3.1).

## 2. Existing designs
- Armored Core: AP (one pool) plus ACS strain/overload (stagger meter, then big damage window); part loadout dictates stats, not per-part HP. https://en.wikipedia.org/wiki/Armored_Core . Takeaway: separate "stagger/overload" meter from health.
- MechWarrior / BattleTech: hit-location table (head, CT, side torsos, arms, legs), armour+internal structure per location, crits to components, destroyed arm loses its weapons, destroyed leg limits movement/falls, torso destruction = kill. https://en.wikipedia.org/wiki/BattleTech . Takeaway: best template for "tankiness = part HP", lethal vs non-lethal parts, cascading loss.
- Real Steel / Rock'em Sockem: boxing robots; damage is cumulative, arm damage degrades punches. Little published model; treat as inspiration only.
- Robot Wars / BattleBots: weapon vs armour, disabling (immobilised, drive lost) is the usual win, not destruction. https://en.wikipedia.org/wiki/Robot_Wars_(TV_series) . Takeaway: "disabled" state (part works at 0%) distinct from "destroyed".
- Gundam battle games (e.g. Extreme VS, Battle Operation): part-break on legs/arms in some titles; mostly HP + down meter. Low confidence on specifics.
- Absolver / Mordhau / For Honor: directional attacks and blocks, weapon swing arcs, hit location by swing; Mordhau uses swept weapon collision with per-limb damage multipliers. https://mordhau.com . Takeaway: swept segment + hit direction is the cheap, readable approach.
- Sekiro: posture bar; blocking deflects fill the enemy's posture; full = deathblow window. https://en.wikipedia.org/wiki/Sekiro:_Shadows_Die_Twice . Takeaway: posture = stagger meter; clashes (weapon on weapon) shift posture.
- Toribash: joint states (contract/extend/hold/relax) on a ragdoll, turn-based; damage from impact force on body parts; well-known for emergent physics. https://www.toribash.com
- Robomaster (DJI): armour plates with hit sensors, damage by plate impact; plate = hitbox. https://www.robomaster.com . Takeaway: hitboxes as discrete plates is a clean sensing model.
- Common pattern: part HP; thresholds (e.g. <50% degraded, 0 disabled); lethal parts (torso/core/head); repair mostly absent in fighters, present between rounds (guess: allow slow regen of disabled->degraded only for sim stability).

## 3. Simple physical models for a dependency-free Rust 2D sim
- Body: tree of rigid links (torso root; head, 2x upper arm, forearm, hand, 2x thigh, shin). Joint angle, angular velocity, torque (motor with target angle PD controller, clamped to max torque scaled by part health).
- Kinematics: forward kinematics each tick from joint angles (no full rigid-body solver). Simpler and deterministic. Full dynamics (Featherstone/Box2D style) is heavier; skip unless needed. Ref: https://box2d.org (as a reference design, not dependency).
- Strike: hand tip velocity v from FK (finite difference). Effective mass m_eff = sum of masses of links in the active chain weighted by alignment; impact = 0.5 * m_eff * (v . n)^2 where n is the contact normal/strike direction. Alignment term: cosine between each link's velocity and the strike direction; torso twist and hip add velocity at the shoulder.
- Coordination: timing = joint peak velocities should occur in proximal-to-distal order (hip -> torso -> shoulder -> elbow -> wrist), the real "kinetic chain". Score = product of each link's contribution times a timing term (guess). See biomechanics of punching literature (kinetic chain), not fetched.
- Balance: centre of mass (mass-weighted FK positions) projected onto support polygon (feet span in side view; feet disc in top-down). Margin < 0 -> stumble; impulses shift the CoM; knockdown when margin stays negative or both legs disabled. Get-up costs time.
- Collision: capsules (segment + radius) per link; hit = swept capsule of hand/weapon over the tick vs target capsules (segment-segment closest point). O(parts^2) = tiny.
