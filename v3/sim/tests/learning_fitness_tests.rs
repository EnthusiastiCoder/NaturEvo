// Learning-side anti-stalemate fitness: pressure when capped, finishing speed, accuracy, punish and landed variety.
// None of these pays for guarding, blocking or parrying; none is an in-fight rule.
use mech_sim::curriculum::{opponent_mix, OppKind};
use mech_sim::duel::{Brain, Match};
use mech_sim::fitness::*;
