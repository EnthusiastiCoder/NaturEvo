// Assumed API: campaign::{Library::new(Vec<Genome>), Library::pick(i)->&Genome, MapCfg::default(),
// CampaignResult::end_count(), Encounter{kind, zone, tile, arena_seed, match_seed, monster_index, result},
// tier_stats(zone_tier, boss)->TierStats{hp,damage}, Outcome{Won,Died,Capped}, EncounterKind{Grass,Elite,Boss}}.
// Replay rule: hero = library.pick(0) here (single-genome libraries), monster = monster_lib.pick(e.monster_index).
use arena_sim::campaign::{run, tier_stats, EncounterKind, Library, MapCfg, Outcome as CO};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::matchup::{play_with_setup, Outcome, Setup};
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
        let setup = Setup { hp: [cfg.max_hp, e.monster_hp], species: [Species::Hero, e.species], rank: [Rank::Grass, e.rank] };
        let again = play_with_setup(Arena::generate(e.arena_seed, &cfg), h.pick(0).clone(), m.pick(e.monster_index).clone(), e.match_seed, &cfg, setup);
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

#[test]
fn run_file_fight_paths_resolve_relative_to_the_run_file() {
    use arena_sim::campaign::run_json;
    let (h, m) = libs(3);
    let r = run(3, MapCfg::default(), &h, &m, true);
    let base = std::env::temp_dir().join(format!("nv_run_rel_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let run_file = base.join("campaign.json");
    let fights = base.join("fights_on");
    let json = run_json(&r, Some(fights.to_str().unwrap()), Some(run_file.to_str().unwrap())).unwrap();
    assert!(json.contains("\"fight\":\"fights_on/fight_0.json\""), "{}", &json[json.len().saturating_sub(300)..]);
    assert!(base.join("fights_on/fight_0.json").exists());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn wild_monsters_are_weaker_than_elites_and_boss_is_full_strength() {
    let (h, m) = libs(4);
    let (mut g, mut el, mut b) = (0, 0, 0);
    for seed in 0..20u64 {
        for e in &run(seed, MapCfg::default(), &h, &m, false).encounters {
            assert!(e.monster_hp <= 100.0 && e.monster_hp > 0.0);
            assert!(e.result.monster_hp <= e.monster_hp + 1e-3, "monster exceeded its starting hp");
            match e.kind {
                EncounterKind::Grass => { assert!(e.monster_hp < 100.0); g += 1; }
                EncounterKind::Elite => { assert!(e.monster_hp < 100.0); el += 1; }
                EncounterKind::Boss => { assert_eq!(e.monster_hp, 100.0); b += 1; }
            }
        }
    }
    assert!(g > 0 && el > 0 && b > 0);
}

#[test]
fn encounters_record_species_rank_and_name_by_kind() {
    let (h, m) = libs(4);
    let (mut g, mut el, mut b) = (0, 0, 0);
    for seed in 0..20u64 {
        for e in &run(seed, MapCfg::default(), &h, &m, false).encounters {
            assert_ne!(e.species, Species::Hero, "monsters are never the hero species");
            assert!(!e.monster_name.is_empty());
            match e.kind {
                EncounterKind::Grass => { assert_eq!(e.rank, Rank::Grass); g += 1; }
                EncounterKind::Elite => { assert_eq!(e.rank, Rank::Elite); el += 1; }
                EncounterKind::Boss => {
                    assert_eq!(e.rank, Rank::Boss);
                    assert!(e.species == Species::Warrior || e.species == Species::Mage, "boss species {:?}", e.species);
                    b += 1;
                }
            }
        }
    }
    assert!(g > 0 && el > 0 && b > 0);
}

#[test]
fn species_rank_and_name_are_deterministic_by_seed() {
    let (h, m) = libs(5);
    let a = run(7, MapCfg::default(), &h, &m, false);
    let b = run(7, MapCfg::default(), &h, &m, false);
    assert_eq!(a.encounters.len(), b.encounters.len());
    for (x, y) in a.encounters.iter().zip(b.encounters.iter()) {
        assert_eq!((x.species, x.rank), (y.species, y.rank));
        assert_eq!(x.monster_name, y.monster_name);
    }
}

#[test]
fn all_four_species_appear_across_seeds() {
    let (h, m) = libs(6);
    let mut seen = [false; 4];
    for seed in 0..30u64 {
        for e in &run(seed, MapCfg::default(), &h, &m, false).encounters {
            match e.species { Species::Minion => seen[0] = true, Species::Rogue => seen[1] = true, Species::Warrior => seen[2] = true, Species::Mage => seen[3] = true, Species::Hero => panic!("hero species monster") }
        }
    }
    assert_eq!(seen, [true; 4]);
}

#[test]
fn fight_and_run_json_carry_species_rank_and_monster_name() {
    use arena_sim::campaign::{fight_json, run_json};
    let (h, m) = libs(3);
    let r = run(3, MapCfg::default(), &h, &m, true);
    let e = &r.encounters[0];
    let fj = fight_json(e);
    assert!(fj.contains(&format!("\"species\":\"{}\"", e.species.as_str())), "{}", &fj[fj.len().saturating_sub(300)..]);
    assert!(fj.contains(&format!("\"rank\":\"{}\"", e.rank.as_str())));
    assert!(fj.contains(&format!("\"monster_name\":\"{}\"", e.monster_name)));
    let rj = run_json(&r, None, None).unwrap();
    for k in ["\"species\"", "\"rank\"", "\"monster_name\""] { assert!(rj.contains(k), "run json missing {k}"); }
}

fn champ_dir(name: &str, hero: &Genome, monster: &Genome) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("nv_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("champion_hero_gen_0.json"), hero.to_json()).unwrap();
    std::fs::write(d.join("champion_monster_gen_0.json"), monster.to_json()).unwrap();
    d
}
}
