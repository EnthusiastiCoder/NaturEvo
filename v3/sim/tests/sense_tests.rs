// W3 sensing: input vector length, finiteness, mirroring, head damage.
mod common;
use common::{arena, standing};
use mech_sim::robot::{Joint, Part, Robot, Role, DT, STAND_TARGETS};
use mech_sim::rng::Rng;
use mech_sim::sense::{observe, N_IN, OPP_HP, OPP_POINTS};
