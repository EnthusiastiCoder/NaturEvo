// W3 fuzz: 200 random-genome 900-tick matches stay finite, in range and never panic.
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::robot::{Part, Role};
use mech_sim::rng::Rng;
