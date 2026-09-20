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

#[test]
fn aggressor_passes_the_screen_thresholds() {
    let s = screen(&aggressor(Role::Hero), Role::Hero, &panel(), 4);
    assert_eq!(s.block_share, 0.0, "{s:?}");
    assert_eq!(s.block_unseen_share, 0.0, "{s:?}");
    assert!(s.draw_rate <= 0.25, "{s:?}");
    assert!(s.statuses_per_fight >= 1.0, "{s:?}");
}

#[test]
fn screen_is_deterministic() {
    let g = turtle(Role::Hero);
    assert_eq!(screen(&g, Role::Hero, &panel(), 3), screen(&g, Role::Hero, &panel(), 3));
}

#[test]
fn from_dir_drops_turtle_and_keeps_normal() {
    let d = tmp_dir("mix");
    put(&d, 5, &turtle(Role::Hero)); // newest first, so a plain truncate(1) would keep the turtle
    put(&d, 3, &aggressor(Role::Hero));
    let lib = Library::from_dir(d.to_str().unwrap(), "hero", 1).unwrap();
    assert_eq!(lib.len(), 1);
    assert_eq!(lib.pick(0).weights, aggressor(Role::Hero).weights);
}
