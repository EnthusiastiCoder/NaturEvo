#!/usr/bin/env bash
# Train three seeds (200 gens, pop 64) into v3/runs/slice_s1..s3 and evaluate them.
set -e
cd "$(dirname "$0")/../sim"
cargo build --release
for s in 1 2 3; do
  ./target/release/mech-sim train --seed $s --pop 64 --gens 200 --out ../runs/slice_s$s
done
./target/release/mech-sim eval --runs ../runs/slice_s1,../runs/slice_s2,../runs/slice_s3 --seeds 4 --out ../runs/eval_seeds.json --md ../runs/eval_slice.md
