#!/usr/bin/env bash
# Style pipeline: distil each scripted teacher into a seed net, co-evolve each style population (round 1), then
# round 2 against the other styles' round-1 champions, then write the style matrix to v3/runs/eval_styles.md.
# GENS defaults to 1000 (about 8 minutes per style on 12 cores).
set -e
cd "$(dirname "$0")/../sim"
cargo build --release
G=${GENS:-1000}
B=./target/release/mech-sim
for s in press counter duelist; do
  $B distil --style $s --out ../runs/seed_$s
  $B train --seed 1 --pop 64 --gens $G --style $s --seed-dir ../runs/seed_$s --out ../runs/style_r1_$s
done
for s in press counter duelist; do
  foes=""
  for o in press counter duelist; do
    [ $o != $s ] && foes="$foes,../runs/style_r1_$o/champion_hero_gen_$((G-1)).json"
  done
  $B train --seed 2 --pop 64 --gens $G --style $s --seed-dir ../runs/style_r1_$s --foes ${foes#,} --out ../runs/style_$s
done
$B styles --teachers --dirs press=../runs/style_press,counter=../runs/style_counter,duelist=../runs/style_duelist --seeds 30 --md ../runs/eval_styles.md
