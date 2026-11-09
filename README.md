# NaturEvo

Deterministic simulations where fighters learn to fight by evolution. Each version is a
self-contained Rust simulation (std only, no third-party crates) with a browser viewer to replay
what the AI learned.

<!-- GIF: two evolved mechs trading sword blows and shield blocks in the v3 viewer. -->
_Demo GIF coming soon._

## Versions

| Folder | What it is | Stack |
|---|---|---|
| [`v1/`](v1/README.md) | **Jungle sim.** A hero levels up through seven jungle zones to defeat the Jungle King: seeded tile map, BFS walking, turn-based fights with per-species move tables, and a static replay viewer with a batch dashboard. | Rust, HTML/JS |
| `v2/` | **Neuroevolution arena.** Every encounter on the jungle map is a real-time 2D duel between co-evolving neural nets (heroes vs monsters) with ray-cast vision, stamina, blocks and dodges, a hall of fame, campaign mode with permadeath, and a record mode for showcase videos. | Rust, PixiJS, Vite, TypeScript |
| `v3/` | **Mech duels.** Two giant humanoid robots with a physical sword and shield fight in a side-view arena. Thirteen body parts each carry HP and armour; damaged parts weaken or remove the moves that depend on them; hits have location, direction and impact (knockback, hit-stop, sparks, stagger). Controllers are evolved neural nets, trained against scripted and style bots. | Rust, PixiJS, Vite, TypeScript |

## Highlights

- Fully deterministic: the same seed gives a byte-identical log, independent of thread count.
- Zero-dependency Rust simulations with multi-threaded training.
- Viewers built for showcase footage: smooth interpolation, a clean record mode and training
  curves (fitness, win rate against the hall of fame, action frequencies).

## Run

Requires Rust (stable) and Node 20+. Run each block from the repo root.

```bash
# v1: see v1/README.md
cd v1 && cargo run --release --manifest-path sim/Cargo.toml -- run --seed 1

# v2: train, then open the viewer
cd v2/sim && cargo run --release -- train --seed 1 --gens 200 --out ../runs/demo
cd ../viewer && npm install && npm run dev

# v3: train a mech controller, evaluate it, then open the viewer
cd v3/sim && cargo run --release -- train --seed 1 --pop 64 --gens 200 --out ../runs/demo
cargo run --release -- eval --runs ../runs/demo --seeds 4
cd ../viewer && npm install && npm run dev
```

## Tests

```bash
cd v3/sim && cargo test
cd v3/viewer && npm test
```

The same commands work in `v1/sim`, `v2/sim` and `v2/viewer`; the v1 viewer checks are
Node scripts in `v1/tests/`.

Plans live in [`plans/`](plans/) and research notes in [`research/`](research/).
Sprite assets in `v2/assets/` are KayKit (CC0); see [`v2/assets/CREDITS.md`](v2/assets/CREDITS.md).

## Copyright

© Arpan Mandal. All Rights Reserved.

See [`COPYRIGHT`](./COPYRIGHT) for the terms. Third-party assets keep their own licences
(see the credits file above).
