# v3 plan: seeded sword-and-shield move vocabulary, learned grace, rule-driven lethality

## Goal
User request (verbatim): "also can we seed in some cool moves from other games? like our ai doesnt use the sword gracefully and usefully. it has to learn the grace and also be lethal and punishing."

Seed a vocabulary of 13 sword/shield moves plus Idle (14 net choices). The net outputs a move choice plus 4 continuous parameters (reach, speed, arc, delay) and a drive, instead of 12 raw joint targets. A deterministic executor turns (move, params) into smooth keyframe joint targets. The AI must LEARN when, how and with what grace to use each move. Lethality and punishment come from GAME RULES (posture break, whiff recovery, commitment, punish windows, guard break), never from fitness bonuses. Grace is emergent (continuous params, timing, footwork spacing, an energy cost on jerk) and is only MEASURED (metrics), not rewarded.
