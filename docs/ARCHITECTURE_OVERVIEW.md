# Architecture Overview

```
core → engine → runtime → runner → api → stages → planners → pipelines → analyze/bench
```

## Summary
Bijux is contract‑first. `bijux-core` defines canonical contracts, IDs, and canonicalization. Planners produce an `ExecutionGraph`. The engine orchestrates execution of that graph without owning execution backends. Runtime defines run layout and recording. Runner provides concrete execution backends (local/docker). The API is the user‑facing orchestration layer. Stages define specs and observers. Pipelines define scientific presets and profiles. Analyze/benchmark read run artifacts and produce reports and comparisons.

## Data Flow (Happy Path)
1. User requests a plan or run via the API.
2. Planner selects tools and produces an `ExecutionGraph`.
3. Engine orchestrates steps using the Runner trait.
4. Runtime records per‑step artifacts and a run manifest.
5. Analyze and benchmark consume the run artifacts to produce reports and comparisons.
