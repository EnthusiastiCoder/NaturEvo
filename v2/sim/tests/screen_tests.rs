// WS3 library screen. Hooks: evolve::screen, Library::from_dir (screens), Library::from_dir_opts(dir, role, keep, screen).
mod common;
use arena_sim::campaign::Library;
use arena_sim::evolve::screen;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use common::*;

fn panel() -> Vec<Genome> { vec![idler(Role::Monster)] }

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("arena_screen_{}_{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}
fn put(dir: &std::path::Path, gen: u32, g: &Genome) {
    let mut g = g.clone();
    g.generation = gen;
    std::fs::write(dir.join(format!("champion_hero_gen_{gen}.json")), g.to_json()).unwrap();
}

#[test]
fn turtle_fails_the_screen_thresholds() {
    let s = screen(&turtle(Role::Hero), Role::Hero, &panel(), 4);
    assert!(s.block_share > 0.25, "{s:?}");
    assert!(s.block_unseen_share > 0.10, "{s:?}");
}
