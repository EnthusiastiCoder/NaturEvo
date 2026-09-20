// WS3 library screen. Hooks: evolve::screen, Library::from_dir (screens), Library::from_dir_opts(dir, role, keep, screen).
mod common;
use arena_sim::campaign::Library;
use arena_sim::evolve::screen;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use common::*;

fn panel() -> Vec<Genome> { vec![idler(Role::Monster)] }
