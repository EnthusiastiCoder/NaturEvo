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

## Verified findings (second pass, fetched/searched 2026-10-24)
1. [V] Mech-X4, https://en.wikipedia.org/wiki/Mech-X4 : 150 ft robot; awakened by pilot Ryan's technopathy; crew roles split (pilot, mechanic, shields/defence operator, weapons operator); S1 remote control from outside the robot; S2 upgrades. Wikipedia gives NO part list or visual/ability detail, so the "humanoid silhouette, glow" claims in section 1 remain UNVERIFIED. Design implication: abilities and shields are crew-style subsystems, so model "shield" and "weapon" as separate energy-using systems, not limbs.
2. [V] Armored Core VI (search results only: Steam discussions, armoredcore.fandom.com, nmiagaming.com): AP = part/mech resilience; impact builds strain on the ACS (attitude control); full strain = stagger; staggered target takes a "direct hit" damage multiplier; impact gauge decays quickly unless explosive/missile. The word "overload" was not confirmed. Design change: make stagger a decaying strain meter with a damage multiplier window (this is the combat payoff), separate from part HP.
3. [V partial] BattleTech (search: AGOAC reference card, https://battletech.com/wp-content/uploads/2025/03/AGOAC-Reference-Card-7th-printing.pdf ; magicgameworld.com intro): 2D6 crit roll: 8-9 one critical location, 10-11 two; destroyed leg = automatic fall; leg/foot actuator damage adds +1/+2 to piloting (balance) rolls; upper/lower leg actuator hit halves kick damage. The Wikipedia page has no such rules (fetch failed to find them); full hit-location table NOT verified. Design change: per-part damage should reduce the specific moves that use it (kick halves), and a balance roll with part-damage modifiers decides falls: use CoM margin penalised per damaged leg part.
4. [V] Sekiro, https://en.wikipedia.org/wiki/Sekiro:_Shadows_Die_Twice : attacks and deflects target posture/balance; broken posture opens a single killing-blow. Confirms the posture-meter idea.
5. [V] Toribash, https://en.wikipedia.org/wiki/Toribash : turn-based, per-joint states relax/hold/contract/extend, neck, pecs, elbows, knees, lumbar, chest, hands grab/release. Damage model NOT verified. Design change: a 4-state joint command (relax/hold/contract/extend) is a compact discrete action alternative to continuous targets.
6. [V] OpenAI robosumo, https://arxiv.org/abs/1710.03748 : competitive self-play yields a natural curriculum; behaviours (sprinting, obstructing, ducking, kicking, defending with arms and legs) far more complex than the simple environment. Reward shaping/annealing details were NOT readable (PDF failed), so the "dense reward annealed" claim stays UNVERIFIED. Design change: lean on hall-of-fame self-play as the curriculum; keep the arena rules simple.
7. [V] Karl Sims, https://www.karlsims.com/evolved-virtual-creatures.html : populations of several hundred, selection plus mutation/crossover of genes encoding growth; tasks include competing for a cube; page shows "bloopers" of cheating exploits. Controller encoding and fitness formulas NOT verified. Design change: budget for physics exploits; add velocity/impulse caps and a "replay review" of top champions.
8. [V] Open-source search: no repo found matching 2D + articulated + fighting + neuroevolution. Closest: AI-Kido https://github.com/PRMorgan/AI-Kido (NEAT, Pygame, no ragdoll), TU Wien thesis on neuroevolution for 2D physics-based character animation with Box2D in Unity https://repositum.tuwien.at/handle/20.500.12708/7429 (positive result, real-time use proposed as future work). Conclusion: v3 has little prior art to copy; expect to be exploring.
