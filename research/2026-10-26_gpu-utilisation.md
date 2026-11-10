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
