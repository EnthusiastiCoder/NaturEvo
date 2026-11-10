# Sword-and-shield move vocabulary for a side-view 2D mech (research, 2026-10-24)

Method: web search summaries plus a few fetched pages. Frame-level data was thin. Anything without a URL, or marked UNVERIFIED, is design reasoning, not a sourced fact.

## 1. Source survey

### Sekiro
- Sources: https://steamcommunity.com/app/814380/discussions/0/1850323802590442995, https://www.gamespot.com/gallery/sekiro-guide-14-best-tips-to-help-you-survive/2900-2656/
- Moves: deflect, perilous attack (thrust / sweep / grab, marked by a red kanji), mikiri counter, jump-kick counter, deathblow.
- Trigger: deflect = guard pressed just before contact. Mikiri = dodge toward the enemy as the thrust lands.
- Timing: no frame data found, so deflect and mikiri windows are UNVERIFIED. Sources say only that the thrust deflect is "incredibly precise".
- Counters:
  - Thrust: mikiri, which does big posture damage.
  - Sweep: jump, then jump-kick for posture damage.
  - Grab: step-dodge.
  - Sweep and grab cannot be deflected.
- Punish rule:
  - Posture fills from attacks and deflects, and the guard breaks when it is full. A broken guard allows a deathblow.
  - Posture regenerates quickly, so chip-and-retreat does not work. Pressure is required.
  - A mikiri only leads to a deathblow if it fills posture.

### Elden Ring / Dark Souls
- Sources: https://eldenring.wiki.gg/wiki/Skill:_Golden_Parry, https://eldenring.wiki.gg/wiki/Skill:_Buckler_Parry, https://eldenring.wiki.gg/wiki/Skill:_Parry
- Parry windows (Elden Ring wiki):
  - Golden Parry: 3 frames (0.10 s), starting at 0.17 s.
  - Buckler Parry: 5 frames (0.17 s), starting at 0.17 s.
  - Standard Parry: 7 frames (0.23 s), starting at 0.13 s.
- Mistimed parry: becomes a partial block with heavy stamina drain and some damage.
- Successful parry: stagger, then a critical riposte. Bosses often need 2-3 parries to break stance.
- Guard break: heavy attacks, guard counters (block, then heavy), and hammer-type weapons do extra stamina damage to blockers.
- Backstab: circle behind the target. Following a riposte or backstab with a heavy is advised.
- Dark Souls frame data: not found, UNVERIFIED.

### For Honor
- Sources: https://forhonor.fandom.com/wiki/Guard_Break, https://www.ubisoft.com/en-us/game/for-honor/news-updates/1IZoiczWorTTTsrEYprgzR/public-test-meta-changes, https://steamcommunity.com/app/304390/discussions/0/1709564547929650327, https://www.gamepressure.com/forhonor/attacks/z796b6
- Moves: light and heavy attacks, guard directions, feint (cancel a heavy), guard break, counter guard break, parry. Chains, revenge and punishes were not covered by the sources I found and are UNVERIFIED.
- Timing:
  - Light attack about 500 ms (one character).
  - Heavy about 600-800 ms. The 800 ms figure comes from the guard-break wiki, which says heavies faster than 800 ms are guaranteed after a guard break.
  - Guard break 400 ms, or 300 ms after a feint. The counter-guard-break sweetspot is 300 ms.
  - Lights are not feintable. These figures are from an older wiki and may be outdated.
- Counters: parry (matching direction), guard break, counter guard break.
- Punish rule:
  - Every light is guaranteed after a guard break.
  - A parry grants a punish window.
  - The 2017 patch made parrying too early punishable, and reduced light-parry advantage to match heavy-parry advantage.
  - The designers wanted parries to pay off without making attacks feel too risky.

### Mordhau
- Sources: https://www.gamepressure.com/mordhau/combat/zac3ca, https://mordhau.fandom.com/wiki/Combat?oldid=3196, https://primagames.com/gaming/how-to-1v1-in-mordhau
- Moves:
  - Feint: cancel during windup. One wiki page puts the cutoff at the last 50 ms of windup. It costs stamina.
  - Morph: switch swing to stab, or the reverse, mid-windup. The window is smaller than the feint window.
  - Drag / accel: change when the hit lands by turning. A drag delays the hit, an accel brings it earlier. Over-dragging reduces damage.
  - Riposte: a fast attack right after a parry. It is not guaranteed.
  - Chamber: an advanced parry that is also an attack. Its timing windows were not found, UNVERIFIED.
  - Kick: UNVERIFIED, not in the sources I found.
- Counters: parry, chamber, morph against parry-heavy opponents, and feints against reactive parries.
- Punish rule: feint and drag exist to bait and mistime the parry. A parry's reward is the riposte.

### Chivalry
- Not researched. UNVERIFIED.

### Toribash
- Sources: https://www.linuxlinks.com/Toribash/, https://tvtropes.org/pmwiki/pmwiki.php/VideoGame/ToriBash
- Joint states: relaxed, held, contracting, expanding, set per turn.
- Damage is scored in points. Some body parts are worth more points and some limbs break more easily.
- Hard blows can break or detach limbs.
- The formula from joint velocity is not published in the pages found. UNVERIFIED.
- Design takeaway (reasoning, not sourced): effective strikes come from sequencing joint extension and momentum, so a mech move should be a joint-target timeline.

### HEMA (Liechtenauer tradition)
- Sources: https://en.wikipedia.org/wiki/Johannes_Liechtenauer, https://akban.org/wiki/German_school_of_swordsmanship, https://thearma.org/forum/viewtopic.php?p=8315
- Guards (Leger): Vom Tag (sword above the shoulder or head), Alber, Ochs (point to the face, hilt by the head), Pflug (hilt at the hip, point to the face).
- Master cuts:
  - Zornhau: a diagonal cut from Vom Tag that binds strong-on-strong.
  - Krumphau.
  - Zwerchhau: a horizontal cut.
  - Schielhau.
  - Scheitelhau: a vertical cut.
- Winden, other techniques and principles:
  - Winden are binds and turns. Counts vary by source (8 or 24).
  - Also: Versetzen (displacement), Absetzen, Durchwechseln, Zucken, Abschneiden, Händedrücken.
  - Vor / Nach, strong and weak, and Indes ("in that instant" of blade contact, when feeling decides the next move).
- Not sourced: half-swording, pommel strike, sword-and-buckler, and tempo specifics. All UNVERIFIED here.
- Punish rule: in HEMA it is tempo-based. Strike while the opponent's blade is displaced or committed. A displacement that also strikes (Versetzen) has the best tempo. This tempo logic is general HEMA principle, reasoning-level here.

## 2. Rules that make play lethal and punishing

| Rule | Typical numbers | Source |
|---|---|---|
| Parry window is short | 3-7 frames, 0.10-0.23 s (Elden Ring) | Elden Ring wiki above |
| Failed or mistimed parry | partial block, stamina drain, some damage | Elden Ring wiki above |
| Posture / stance break leads to a finisher | full gauge, then deathblow or critical | Sekiro, Elden Ring |
| Posture regen blocks chip-and-run | "bosses regain posture quickly" | Sekiro, steamcommunity link above |
| Guard break | 300-400 ms stun. Lights guaranteed, heavies under 800 ms guaranteed | For Honor wiki above |
| Feint is cheap but not free | cutoff in last ~50 ms of windup, stamina cost | Mordhau wiki above |
| Lights not feintable | lights are safer, heavies are bait-able | For Honor 2017 patch above |
| Early parry punished | patch made early parries punishable | For Honor 2017 patch above |
| Riposte is not guaranteed | needs correct timing | Mordhau, gamepressure link above |
| Attack-type counters | thrust: mikiri. Sweep: jump. Grab: step | Sekiro |

Whiff recovery and commitment cost: no sourced numbers. For this project, derive recovery as a fraction of total move time, plus stamina. Make recovery frames non-cancellable except by a feint, which costs stamina.

## 3. Grace metrics (proxies)

These are reasoning-based proxies, not sourced thresholds.

- Jerk (mean squared third derivative of the tip or joint angles): smooth, committed arcs score low. Biomechanics literature uses minimum-jerk as the smoothness model. Source not fetched, UNVERIFIED.
- Spacing consistency: variance of distance-to-opponent at the start of attacks, relative to the move's reach. This matches the idea that attacks work from the right range, and the 'Vor/Nach' distinction (Wikipedia link above).
- Hits per swing: a hit-rate measure. Reward only when the hit is not a trade.
- Whiff rate: swings that connect with nothing. A direct commitment-cost proxy.
- Punish rate: share of opponent recovery windows that are hit. Taken from the punish-window rules above.
- Recovery efficiency: time from the end of the active phase to the next guard-ready state, relative to the move's nominal recovery.
- Rule: use these for evaluation and logging only. Lethality should come from the rules in section 2, not from fitness terms.

## 4. Recommended vocabulary (14 moves)

Time is in seconds. W = windup, A = active, R = recovery. Reach is a fraction of arm plus sword length.

| # | Move | Origin | Params (reach / speed / arc / timing) | Joints | W / A / R | Counter | Punish rule |
|---|---|---|---|---|---|---|---|
| 1 | Overhead cut | Scheitelhau; Souls vertical | 0.9 / medium / 120 deg descending / commits late | R shoulder, elbow, waist | 0.35 / 0.10 / 0.40 | guard, parry, backstep | Whiff leaves a long recovery, a free punish. A hit on a guard adds big posture damage. |
| 2 | Diagonal descending | Zornhau | 0.9 / medium / 90 deg / angle param | R shoulder, elbow, waist | 0.30 / 0.10 / 0.35 | guard, parry | Slightly shorter recovery than overhead. |
| 3 | Diagonal rising | Krumphau-style; Mordhau upswing | 0.8 / fast / 90 deg ascending | R shoulder, elbow, waist | 0.25 / 0.10 / 0.30 | low guard, backstep | Cheap but low damage. |
| 4 | Horizontal sweep | Zwerchhau; Sekiro sweep | 1.0 / medium / 150 deg / wide | R arm, waist twist | 0.30 / 0.15 / 0.45 | jump or back out of range, parry | Wide arc, worst whiff recovery. |
| 5 | Thrust / lunge | Pflug; Sekiro thrust | 1.2 / fast / linear / forward step | R arm, hips, knees | 0.20 / 0.08 / 0.35 | mikiri, parry, sidestep | Longest reach. A mikiri hit gives heavy posture damage. |
| 6 | Feint | Mordhau feint; For Honor feint | same as move it cancels / cancel in last ~50 ms | as the cancelled move | W then cancel / 0 / 0.10 | reactive parry | Costs stamina. Baits a parry, then a punish window. |
| 7 | Parry | Elden Ring / Sekiro deflect | window 0.10-0.23 s / angle match | sword arm, shield arm | 0.05 / 0.10-0.15 / 0.25 | feint, morph | A success staggers the attacker. A miss means a partial block and stamina loss. |
| 8 | Riposte | Mordhau, Souls | fast follow-up / only after a successful parry | R arm | 0.15 / 0.10 / 0.30 | none while staggered | Guaranteed on a parried target, so it is the lethal payoff. |
| 9 | Guard | Shield block | shield-side angle | L shoulder, elbow | 0.10 / hold / 0.10 | guard break, thrust to the side | Blocked damage fills posture, so it is not free. |
| 10 | Shield bash / guard break | For Honor guard break | 0.5 / fast / forward | L arm, waist | 0.25 / 0.10 / 0.35 | backstep, parry | Breaks a guard for 0.3-0.4 s, then a free light. |
| 11 | Low sweep / leg cut | Krumphau; Sekiro low | 0.9 / medium / low arc | R arm, waist, knees | 0.30 / 0.12 / 0.40 | hop, guard low | Hits legs, so it costs the target mobility. |
| 12 | Drag / follow-up chain | Mordhau drag | extra delay param on a swing / retimed hit | any cut | +0.1-0.2 on W | early parry | Mistimes parries. Costs stamina. |
| 13 | Pommel strike / kick | HEMA pommel; Sekiro kick | 0.4 / fast / short | R wrist or hip | 0.15 / 0.08 / 0.30 | backstep | Close-range posture hit. Short reach. |
| 14 | Backstep + counter | Mikiri / HEMA tempo | reach 0.0 to 1.0 / fast / retreat then strike | hips, knees, ankles | 0.20 / 0.10 / 0.25 | chase | Punishes the whiff of a committed long move. |

Mikiri-style anticipation counter: folded into row 14 plus the thrust counter, not a separate move. A dedicated one would need a window that does not exist in the sources I found.

Numbers in rows 1-14 are design proposals, not sourced values, except the guard-break, parry and feint figures in section 2.
