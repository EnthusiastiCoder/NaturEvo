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

## 4. Neuroevolution of articulated fighters
- Evolved virtual creatures (Sims 1994): morphology+controller coevolved; exploits physics bugs. https://www.karlsims.com/evolved-virtual-creatures.html
- OpenAI robosumo (Bansal et al. 2018): emergent complexity via self-play with competition; used curriculum of dense shaping reward (stay in ring, near opponent) annealed to win/lose. https://arxiv.org/abs/1710.03748
- DeepMind humanoid soccer / locomotion: https://arxiv.org/abs/2105.12196 (multi-agent soccer), https://arxiv.org/abs/1707.02286 (locomotion in rich environments). Lessons: population + self-play, reward shaping decayed over time.
- Toribash AI: community bots exist; little published.
- IO: joint angles and velocities (normalised), part health fractions, contact/touch flags per capsule, opponent part positions relative to own torso in own frame (not world), CoM margin, stamina/posture. Outputs: target joint angles or torque per joint in [-1,1], plus optional discrete "intent" outputs.
- Pitfalls: flailing (reward only impact * quality, penalise energy and require alignment), turtling (stagger/stalemate damage ticking, ring shrink), stuck on ground (get-up reflex or short fall timeout), physics exploits (clamp velocities, cap impulses), fitness rewarding self-damage. Curriculum: stand -> approach -> hit static dummy -> sparring vs past champions (hall of fame, as v2 does).
- Size: ~14 joints x 2 + senses ~ 60-80 inputs, 14-18 outputs; one hidden layer of 24-48 with tanh or small recurrent unit (guess). Per-tick cost ~ 80*32 + 32*16 mults, fine. Determinism: seeded RNG, fixed dt, no float-order variance across threads, avoid HashMap iteration.
- Uncertainty: no source gives a definitive net size; my number is an estimate, tune empirically.

## 5. View and rendering
- Side view reads articulated limbs, punch height, falls, balance far better; top-down hides limb pose and balance. For "coordinated parts" choose side view (2D platform fighter style). Top-down only if the arena's movement matters more than limb pose.
- Rendering: vector-drawn capsules/rounded polygons per part, outline plus darker back-limbs for depth, additive glow on core/eyes (Canvas 2D `globalCompositeOperation='lighter'`, shadowBlur sparingly), spark particles on contact in impact direction, brief hit-stop (3-6 frames), screen shake scaled by impact, part damage cracks/colour desaturation, destroyed part detaches as a falling body. Canvas 2D is enough for 2 robots; WebGL not needed.

## Design sketch
Parts (roles): head (sensors; head 0% = blind-ish: noisier opponent input, lethal at 0), torso (core; lethal; twist joint), pelvis/hips (power + balance), upper arm L/R (reach, power), forearm L/R, hand/weapon L/R (damage carrier), thigh L/R, shin/foot L/R (balance, kick).
Moves (required parts -> quality):
- Jab: shoulder, elbow, hand; quality = reach alignment * elbow speed.
- Cross/hook: + torso twist + hip; kinetic-chain order bonus.
- Kick: hip, knee, foot, plus opposing leg for balance; failure if balance margin < 0.
- Block: forearm/shield; absorbs by blocking-part HP, fills attacker posture on clash.
- Dash/shove: legs + torso.
Quality = mean-weighted(joint contributions each scaled by part health) * timing_order * alignment * balance_factor. Disabled required part -> move unavailable; degraded -> proportionally weaker.
Damage: impact = 0.5*m_eff*(v.n)^2 (capped); armour reduction per part; HP per part; thresholds: <50% degraded (torque cap scaled), 0 disabled (limp joint), torso/head 0 = KO. Stagger: posture meter per robot, impact adds, over threshold -> stumble/knockdown. Reaction: impulse on struck link along hit direction.
Per-limb state (12): joint angle, angular velocity, target angle, max torque, health fraction, disabled flag, mass, contact flag, last-impact magnitude, last-impact direction (x,y), local tip speed, armour.
Sensors (~70): per-limb angle/vel/health/contact, CoM margin, posture, stamina, opponent torso/hand/foot relative position and velocity, opponent striking-arm state, distance, wall distance.
Actions (~16): target angle per joint (hips, torso twist, shoulders, elbows, knees, ankles, head), plus brace/guard bias.

## Open questions (recommended answer)
1. Side view or top-down? Side view.
2. Full rigid-body physics or FK-driven joints? FK with PD motors, deterministic.
3. Fixed identical morphology or evolvable body? Fixed humanoid first; vary mass/armour only.
4. Evolve joint targets directly or discrete move menu? Direct joint control; moves are scored post hoc.
5. Allow repair? No in-fight; reset between rounds.
6. Abilities (Mech-X4 powers) in v3.0? No, defer to v3.1 after strikes/balance work.
