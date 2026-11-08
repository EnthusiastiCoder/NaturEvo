// r7: a sword that is not striking cannot rest inside the opponent's body (same rule both sides).
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::stylebot::StyleId;

fn net(path: &str) -> Option<Genome> {
    let p = format!("{}/../runs/{}", env!("CARGO_MANIFEST_DIR"), path);
    std::fs::read_to_string(p).ok().and_then(|s| Genome::from_json(&s).ok())
}

fn violations(a: Brain, b: Brain, seed: u64) -> u32 {
    let mut m = Match::new(seed, a, b);
    let mut bad = 0;
    for _ in 0..2500 {
        let t0 = m.tick;
        if m.is_over() {
            break;
        }
        m.step();
        for i in 0..2 {
            if !m.striking(i) && m.sword_in_body(i) {
                bad += 1;
            }
        }
    }
    bad
}
