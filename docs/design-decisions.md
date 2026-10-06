# Design Decisions: Rust Load Balancer Re-test

Status: **open**. Settle each decision and record the choice and its rationale *before* running the main experiment. Changing any of them after seeing results is the benchmark equivalent of data leakage.

## Baseline from the dissertation (for reference)

- Algorithms: Round Robin, Least-Loaded, smooth Weighted Round Robin, Adaptive.
- Simulation setup: 5 nodes, Poisson arrivals at 2.0/s, heterogeneous capacity normalised to 3× the arrival rate, 600 s simulated runs, 20 paired seeds, paired t-test plus Wilcoxon, p < 0.05, no multiple-comparison correction.
- Adaptive design: a weighted score over load, utilisation and rate, plus three modes switched on average-load thresholds of 2.0 and 10.0. Spare-capacity mode weights rate at 0.6 and load at 0.3. Failed nodes are excluded.
- Heterogeneous results:
  - Throughput: LL, WRR and Adaptive were effectively tied (largest gap about 0.1%). RR lost about 10% from a 198-task backlog behind a slow node.
  - Fairness (Gini): LL 0.526, WRR 0.626, RR 0.658, Adaptive 0.731. Adaptive put 72.5% of tasks on the fastest node.
  - Mode switching: Adaptive was in spare-capacity mode for 99.96% of 23,980 samples.
- **Wording fix:** the fairness penalty is **+0.20 Gini (absolute)**, about **39% worse relative to Least-Loaded**. It is not a "20% penalty".

## Decisions to make

### D1. Fairness metric
Gini over raw request counts penalises sending more work to faster backends, so smooth WRR is "unfair" by construction. This partly explains Adaptive's poor fairness score.
- Option A: Gini over request counts. Directly comparable with the dissertation.
- Option B: Gini over utilisation (requests relative to capacity, or busy time). Measures balance relative to what each backend can handle.
- Option C: report both, with one named as primary in advance.
- **Decision:** _TBD_

### D2. Observable signals for Adaptive
A real load balancer cannot see backend queue length or busy time.
- Load: in-flight requests per backend, counted by the LB.
- Rate: either configured statically or estimated with an EWMA of response times. Decide which, and the EWMA α.
- Utilisation: drop it, approximate it from latency, or have the backend report it (e.g. a stats endpoint). Each changes what "Adaptive" means.
- **Decision:** _TBD_

### D3. Threshold calibration
The 2.0/10.0 thresholds were in simulation units and will not transfer. Calibrate them from a pilot run's load distribution (dissertation §7.4), then freeze them before the main experiment. Log the active mode over time so mode occupancy can be checked, the same telemetry that exposed the 99.96% result.
- **Decision:** _TBD_

### D4. Pairing without determinism
Real runs are not bit-for-bit reproducible.
- Use an **open-loop** load generator that replays the same seeded Poisson arrival schedule for every algorithm.
- Avoid closed-loop tools (e.g. `wrk`) as the primary source, because coordinated omission hides queueing delay.
- Pair by seed and run order. Randomise or counterbalance algorithm order to control for drift (thermal effects, background load).
- **Decision:** _TBD_

### D5. Primary metrics
Response time was deferred in the dissertation (CR1) but is cheap to collect in a real proxy. It is also the fairest test of whether a speed-favouring mode is a design flaw or was judged on the wrong objective (§7.5).
- Candidates: throughput, fairness (per D1), and latency p50/p99.
- Pre-specify the multiple-comparison correction (e.g. Holm or Bonferroni).
- **Decision:** _TBD_

## Benchmark hygiene checklist
- [ ] Warm-up period discarded, with its length fixed in advance.
- [ ] Load generator not co-located with the LB or backends, or the co-location documented and its effect measured.
- [ ] Backends heterogeneous in a controlled, documented way (e.g. injected service delay or CPU limits), with total capacity normalised (NFR3 equivalent).
- [ ] Any nginx/HAProxy comparison uses a tuned config, committed to the repo.
- [ ] A release build is used for all measurements.
- [ ] Raw per-request data saved, not just summaries (FR4 equivalent).
