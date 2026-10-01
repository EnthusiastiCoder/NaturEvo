// W3 determinism: same seed => byte-identical log.
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::log::fight_json;
use mech_sim::rng::Rng;
use mech_sim::robot::Role;

fn run(seed: u64, gseed: u64) -> String {
    let a = Genome::random(Role::Hero, &mut Rng::new(gseed));
    let b = Genome::random(Role::Monster, &mut Rng::new(gseed + 1));
    let mut m = Match::new(seed, Brain::Net(a), Brain::Net(b));
    m.enable_log();
    m.run_to_end();
    fight_json(&m)
}
