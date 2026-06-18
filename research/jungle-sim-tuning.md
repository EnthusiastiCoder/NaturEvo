# Jungle sim balance tuning (WS4)

Baseline: win rate 100%, avg deaths 20.3, avg level 39.7.

Findings (300 runs each, release):
- Global monster_stat multiplier 1.1/1.2/1.3/1.5: 0% wins (the multiplier also scales the boss ~4.4x; hero is stuck retrying at level 40, deaths 14k-20k).
- Boss-only tuning (data.rs Jungle King), hp/atk -> win rate: 480/38 96.7%, 500/38 81%, 480/39 61.7%, 500/39 42.7%, 520/40 1.3%, 560/42 0%.
- The curve is a cliff: boss atk +1 moves win rate ~20-40 points.

Chosen: Jungle King hp 420 -> 480, atk 38 -> 39 (only change, in sim/src/data.rs).
Result: 500 runs ~62%; 2000 runs 64.0%, avg deaths 5020, avg level 40.0.

Caveat: avg deaths is high (~5000) because losing runs retry the boss until the 300k event cap. Failed runs dominate the mean; the 2000-run batch takes ~24s, above the "seconds" target. Capping retries in sim.rs (out of scope here) would fix both.
