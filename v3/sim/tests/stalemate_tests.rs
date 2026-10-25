//! Mutual stagger must not be a stable state: clashes cannot keep both fighters down for a large share of a fight.
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::robot::State;
