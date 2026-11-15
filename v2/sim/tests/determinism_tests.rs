use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::matchup::play;
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

fn run(seed: u64) -> (String, u32) {
    let cfg = ArenaConfig::default();
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    let res = play(Arena::generate(seed, &cfg), h, m, seed, &cfg);
    (res.log.to_json(), res.ticks)
}

#[test]
fn same_seed_gives_byte_identical_log() {
    for seed in 1u64..=100 {
        assert_eq!(run(seed), run(seed));
    }
}

#[test]
fn different_seeds_differ() {
    assert_ne!(run(1).0, run(2).0);
}

#[test]
fn log_has_a_frame_per_tick_and_tick_hz() {
    let (j, ticks) = run(5);
    let n = j.matches("\"hp\"").count();
    assert!(n as u32 >= ticks, "frames {n} ticks {ticks}");
    assert!(j.contains("\"tick_hz\":30") || j.contains("\"tick_hz\": 30"));
}

#[test]
fn scripted_status_match_is_byte_identical() {
    use arena_sim::fighter::Action;
    use arena_sim::matchup::Match;
    use arena_sim::status::Status;
    let go = || {
        let cfg = ArenaConfig::default();
        let mut r = Rng::new(3);
        let h = Genome::random(Role::Hero, &mut r);
        let m = Genome::random(Role::Monster, &mut r);
        let mut mt = Match::new(Arena::generate(3, &cfg), h, m, 3, &cfg);
        mt.place_for_duel(1.0);
        mt.apply_status(Role::Hero, Status::Confused, 90, 1.0, Role::Monster);
        mt.apply_status(Role::Monster, Status::Bleed, 90, 1.0, Role::Hero);
        for i in 0..200 {
            let a = Action { fwd: 1.0, strafe: 0.5, turn: 0.3, attack: i % 7 == 0, attack_kind: (i % 3) as u8, ..Action::idle() };
            mt.step_with(a, Action { attack: true, attack_kind: (i % 2) as u8, ..Action::idle() });
        }
        mt.log().to_json()
    };
    assert_eq!(go(), go());
}
