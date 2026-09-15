// Assumed API: campaign::{Library::new(Vec<Genome>), Library::pick(i)->&Genome, MapCfg::default(),
// CampaignResult::end_count(), Encounter{kind, zone, tile, arena_seed, match_seed, monster_index, result},
// tier_stats(zone_tier, boss)->TierStats{hp,damage}, Outcome{Won,Died,Capped}, EncounterKind{Grass,Elite,Boss}}.
// Replay rule: hero = library.pick(0) here (single-genome libraries), monster = monster_lib.pick(e.monster_index).
use arena_sim::campaign::{run, tier_stats, EncounterKind, Library, MapCfg, Outcome as CO};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Outcome};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

fn libs(seed: u64) -> (Library, Library) {
    let mut r = Rng::new(seed);
    (
        Library::new(vec![Genome::random(Role::Hero, &mut r)]),
        Library::new(vec![Genome::random(Role::Monster, &mut r)]),
    )
}

#[test]
fn deterministic_by_seed() {
    let (h, m) = libs(1);
    let a = run(5, MapCfg::default(), &h, &m, true);
    let b = run(5, MapCfg::default(), &h, &m, true);
    assert_eq!(a.steps, b.steps);
    assert_eq!(a.encounters.len(), b.encounters.len());
    for (x, y) in a.encounters.iter().zip(b.encounters.iter()) {
        assert_eq!(x.arena_seed, y.arena_seed);
        assert_eq!(x.result.log.to_json(), y.result.log.to_json());
    }
}

#[test]
fn permadeath_ends_on_first_hero_loss_with_exactly_one_end() {
    for seed in 0..10u64 {
        let (h, m) = libs(seed);
        let r = run(seed, MapCfg::default(), &h, &m, true);
        let losses = r.encounters.iter().filter(|e| e.result.outcome != Outcome::HeroWin).count();
        if r.outcome == CO::Died {
            assert_eq!(losses, 1, "seed {seed}");
            assert_ne!(r.encounters.last().unwrap().result.outcome, Outcome::HeroWin);
        } else {
            assert_eq!(losses, 0);
        }
        assert_eq!(r.end_count(), 1);
    }
}

#[test]
fn without_permadeath_run_continues_after_a_loss() {
    let (h, m) = libs(2);
    let mut saw_continue = false;
    for seed in 0..20u64 {
        let r = run(seed, MapCfg::default(), &h, &m, false);
        let first_loss = r.encounters.iter().position(|e| e.result.outcome != Outcome::HeroWin);
        if let Some(i) = first_loss {
            if i + 1 < r.encounters.len() { saw_continue = true; }
        }
        assert_eq!(r.end_count(), 1);
    }
    assert!(saw_continue);
}

#[test]
fn each_encounter_has_replayable_log() {
    let (h, m) = libs(3);
    let r = run(3, MapCfg::default(), &h, &m, true);
    assert!(!r.encounters.is_empty());
    let cfg = ArenaConfig::default();
    for e in &r.encounters {
        assert!(e.result.ticks >= 1);
        assert!(e.result.log.to_json().contains("\"frames\""));
        let again = play(Arena::generate(e.arena_seed, &cfg), h.pick(0).clone(), m.pick(e.monster_index).clone(), e.match_seed, &cfg);
        assert_eq!(again.log.to_json(), e.result.log.to_json());
    }
}

#[test]
fn encounters_have_valid_kinds() {
    let (h, m) = libs(4);
    let r = run(4, MapCfg::default(), &h, &m, false);
    assert!(r.steps > 0);
    assert!(r.encounters.iter().filter(|e| e.kind == EncounterKind::Boss).count() <= 1);
    assert!(r.encounters.iter().any(|e| e.kind == EncounterKind::Grass));
}

#[test]
fn zone_tier_raises_monster_stats() {
    let (t0, t2, boss) = (tier_stats(0, false), tier_stats(2, false), tier_stats(2, true));
    assert!(t2.hp > t0.hp || t2.damage > t0.damage);
    assert!(boss.hp > t2.hp && boss.damage >= t2.damage);
}

#[test]
fn grass_monsters_are_older_champions_and_boss_is_the_latest() {
    let mut r = Rng::new(9);
    let h = Library::new(vec![Genome::random(Role::Hero, &mut r)]);
    let m = Library::new((0..4).map(|_| Genome::random(Role::Monster, &mut r)).collect());
    let (mut g, mut b) = (0, 0);
    for seed in 0..20u64 {
        for e in &run(seed, MapCfg::default(), &h, &m, false).encounters {
            match e.kind {
                EncounterKind::Grass => { assert!(e.monster_index >= 2, "grass index {}", e.monster_index); g += 1; }
                EncounterKind::Boss => { assert_eq!(e.monster_index, 0); b += 1; }
                _ => {}
            }
        }
    }
    assert!(g > 0 && b > 0);
}
