# Jungle Sim

A deterministic Rust simulation of a hero fighting through jungle zones to defeat the Jungle King, plus a static viewer to replay runs.

## Build
Requires Rust (std only, no dependencies).

    cd sim
    cargo build --release

## Run
From the repo root (paths are relative to it):

    cargo run --release --manifest-path sim/Cargo.toml -- run --seed 1      # writes viewer/data/run.js
    cargo run --release --manifest-path sim/Cargo.toml -- batch --runs 2000 # writes viewer/data/stats.js
