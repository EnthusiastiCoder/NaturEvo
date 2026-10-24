#!/usr/bin/env bash
# Train three seeds (1000 gens, pop 64, engaging SkillBot curriculum) into v3/runs/slice_s1..s3, then evaluate them
# (grace metrics, defence shares, win rate vs earlier champions) into v3/runs/eval_vocab.md. Prints seconds per seed.
set -e
cd "$(dirname "$0")/../sim"
cargo build --release
for s in 1 2 3; do
  t0=$(date +%s)
  ./target/release/mech-sim train --seed $s --pop 64 --gens 1000 --out ../runs/slice_s$s
  echo "seed $s: $(( $(date +%s) - t0 )) s" | tee -a ../runs/train_times.txt
done
./target/release/mech-sim eval --runs ../runs/slice_s1,../runs/slice_s2,../runs/slice_s3 --seeds 4 --out ../runs/eval_seeds.json --md ../runs/eval_vocab.md
