//! Posture: the ready stance is a natural upright fighting stance and fights do not hold a heavy lean.
use mech_sim::duel::{Brain, Match};
use mech_sim::robot::{Joint, Part, Robot, Role, State, DT, STAND_TARGETS};

/// Forward torso tilt in degrees from the capsule geometry (what the viewer draws); positive = toward facing.
fn torso_tilt(r: &Robot) -> f32 {
    let c = r.capsule(Part::Torso);
    ((c.b[0] - c.a[0]) * r.facing).atan2(c.b[1] - c.a[1]).to_degrees()
}

#[test]
fn ready_pose_is_upright_with_slightly_bent_knees() {
    let mut r = Robot::new(Role::Hero);
    for _ in 0..200 {
        r.apply_targets(&STAND_TARGETS, DT);
    }
    let t = torso_tilt(&r);
    assert!((0.0..=8.0).contains(&t), "torso tilt {t} deg: want a small forward lean (0..8)");
    let k = r.joint_angle(Joint::KneeL).to_degrees();
    assert!((6.0..=25.0).contains(&k), "knee {k} deg: want slightly bent (6..25)");
    let head = r.capsule(Part::Head);
    let torso = r.capsule(Part::Torso);
    assert!(head.a[1] > torso.b[1], "head above the torso top");
    assert!((head.a[0] - torso.b[0]).abs() * r.facing.signum() < 0.25, "head not thrust forward");
    // Neck keeps the head over the shoulders.
    assert!(r.joint_angle(Joint::Neck).to_degrees().abs() < 8.0);
}

#[test]
fn standing_robot_stays_upright_and_balanced() {
    let mut r = Robot::new(Role::Hero);
    for i in 0..(5.0 / DT) as usize {
        r.apply_targets(&STAND_TARGETS, DT);
        if i > 100 {
            assert_eq!(r.state(), State::Idle);
            assert!(torso_tilt(&r) < 8.0 && torso_tilt(&r) > -2.0);
            assert!(r.balance_margin() > 0.1, "CoM over the feet, margin {}", r.balance_margin());
        }
    }
}

fn fight_lean(a: Brain, b: Brain, seeds: u64) -> (f32, f32, f32) {
    let (mut sum, mut n, mut over, mut back) = (0.0f32, 0u32, 0u32, 0u32);
    for seed in 0..seeds {
        let mut m = Match::new(seed, a.clone(), b.clone());
        while !m.is_over() {
            m.step();
            for r in &m.robots {
                if r.state() != State::Idle {
                    continue;
                }
                let t = torso_tilt(r);
                sum += t;
                n += 1;
                if t > 15.0 {
                    over += 1;
                }
                if t < -12.0 {
                    back += 1;
                }
            }
        }
    }
    let n = n.max(1) as f32;
    (sum / n, over as f32 / n, back as f32 / n)
}

#[test]
fn scripted_fights_keep_the_torso_near_upright_on_average() {
    for tier in [1u8, 3] {
        let (mean, fwd, back) = fight_lean(Brain::Skill(tier), Brain::Skill(tier), 6);
        println!("tier {tier}: mean {mean:.1} fwd>15 {fwd:.3} back<-12 {back:.3}");
        assert!(mean.abs() < 6.0, "tier {tier} mean tilt {mean}");
        assert!(fwd < 0.08, "tier {tier} leans forward >15deg {fwd} of the time");
        assert!(back < 0.12, "tier {tier} leans back >12deg {back} of the time");
    }
}

#[test]
fn random_nets_keep_the_torso_near_upright_on_average() {
    use mech_sim::genome::Genome;
    use mech_sim::rng::Rng;
    let mut rng = Rng::new(7);
    let (mut sum, mut worst) = (0.0f32, 0.0f32);
    for _ in 0..8 {
        let a = Genome::random(Role::Hero, &mut rng);
        let b = Genome::random(Role::Monster, &mut rng);
        let (mean, _, _) = fight_lean(Brain::Net(a), Brain::Net(b), 1);
        println!("net mean tilt {mean:.1}");
        sum += mean;
        worst = worst.max(mean.abs());
    }
    assert!(worst < 10.0, "worst net mean tilt {worst}");
    assert!((sum / 8.0).abs() < 6.0);
}
