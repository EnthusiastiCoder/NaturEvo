// W4 perf: a 900-tick match under 25 ms. Ignored in debug builds; runs under `cargo test --release`.
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::robot::Role;
use mech_sim::rng::Rng;
use std::time::Instant;

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: cargo test --release --test perf_tests")]
fn nine_hundred_tick_match_under_25_ms() {
    let a = Genome::random(Role::Hero, &mut Rng::new(1));
    let b = Genome::random(Role::Monster, &mut Rng::new(2));
    let mut m = Match::new(1, Brain::Net(a), Brain::Net(b));
    let t = Instant::now();
    let r = m.run_to_end();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    assert!(r.ticks > 0);
    // Per 900 ticks: a fight with no forcing rule may run to the 6000-tick cap.
    let per900 = ms * 900.0 / r.ticks as f64;
    assert!(per900 < 25.0, "{:.1} ms per 900 ticks ({} ticks)", per900, r.ticks);
}
