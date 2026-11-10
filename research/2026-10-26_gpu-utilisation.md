# GPU / CPU utilisation for v3 training (2026-10-26)

## 1. Hardware
- CPU: Intel i5-12450H, 8 cores (4P+4E) / 12 logical threads.
- GPU: NVIDIA RTX 4050 Laptop, 6 GB VRAM, driver 616.92 (plus Intel UHD iGPU). nvidia-smi shows no compute apps; its 22% is desktop/browser.

## 2. How training uses the machine
- evolve.rs `run_jobs`: std::thread::scope, N workers pull the next job from a shared AtomicUsize (work-stealing, one fight per job), results go into a Mutex<Vec> slot (one lock per finished fight, negligible). `--threads` defaults to available_parallelism = 12. Output is identical for any thread count.
- The GPU is used 0%. Training is pure CPU.
- Measured (release binary copy, pop 32, 6 gens, while other agents' jobs were also running, so noisy):
  1 thread 38.7s, 4: 10.5s, 8: 7.2s, 12: 5.1s => 7.6x on 12 threads (about 63% per-thread efficiency, expected with 4 E-cores and SMT; the shared box also hosts other training agents and Brave).
- System CPU sampled at 24-98% (mean ~60%) over 8 s: bursts of 100% during job batches, dips between generations. The dips are the serial phase (make_jobs, selection, breeding, hall of fame, file IO) plus tail effect when the last long fights finish. Not code-bound by locks.
- Where time goes: essentially all in `Match::run_to_end` (sim tick ~6 us, 1 fight = thousands of ticks, each with two small MLP forwards, physics, combat). Serial portions are a small fraction (Amdahl, the dips above).
- Important caveat: several agents train concurrently, so the 12 threads are shared. "Not using the CPU fully" is mostly just contention; the machine is already near saturated during batches.

## 3. GPU options
(a) Batched net inference on GPU: nets are tiny MLPs (tens of inputs, few hundred weights). Per-tick inference is ~1 us of the 6 us tick; a dispatch plus readback per tick costs 20-100 us latency, so it would be SLOWER unless thousands of fights run in lock step. Lock-step needs a restructure (all fights advance one tick together; fights end at different times). Even at zero GPU cost, removing inference saves at most ~15-20%. Verdict: speedup <1x to ~1.2x, high cost, needs wgpu/CUDA dependency and float determinism issues.
(b) Whole sim on GPU (compute shader/CUDA, thousands of matches in parallel): the theoretical big win (pop 64 gives ~hundreds of fights per generation, more with 6 GB VRAM, maybe 20-50x on throughput after rewrite). But: heavy branching (move state machines, combat, stagger, parry), per-fight divergence, f32 only on consumer GPU and non-bit-identical vs the CPU sim (breaks determinism tests, replays in the viewer, existing champions); a second sim implementation must be kept in sync with every mechanic change (the sim is being edited daily); needs wgpu (Rust deps) or CUDA toolkit. Effort: weeks, ongoing maintenance doubled. Also with only ~200-600 fights per generation, the GPU would be under-occupied unless pop is greatly increased.
(c) More CPU parallelism and sim speedups: 
 - Run fewer concurrent agent trainings or give each a thread budget (--threads) so batches are not oversubscribed; oversubscription hurts more than anything. Expected +10-30% in wall time.
 - Overlap the serial phase (breeding/IO) with evaluation, or pipeline hero/monster populations; recovers the dips: about +10-15%.
 - Longest-fight-first job ordering reduces tail: +5-10%.
 - Sim hot loop: avoid per-job `a.clone()`/allocations, fixed-size arrays, f32 nets with SIMD-friendly layout, early termination of dead-draw fights (stall detection): plausible 1.5-2x with profiling. No dependencies needed.
 - Cheap stdlib-only option: build with `-C target-cpu=native` and lto/codegen-units=1 for 5-20%.
