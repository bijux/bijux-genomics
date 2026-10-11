# Watchdog contracts

`watchdog::simulate_dag_watchdog_path` takes repository, config, DAG-report and output paths, a borrowed `BenchmarkLocalPipelineDagValidationReport` and `LocalDagWatchdogScenario`. It admits canonical schema, valid acyclic topology and dependency-first nodes before simulation or filesystem effects. Scenario-specific missing stages and failed invariants return errors. Paths describe caller provenance; the owner does not reread them to authenticate the supplied report.

The result is the original `LocalDagWatchdogSimulationReport` with unchanged node and completion-check records, field names and `bijux.bench.local_dag_watchdog_simulation.v1` identity. Failed input admission preserves an existing output. A successful simulation publishes JSON atomically and returns the same record. Four existing default output constants and scenario labels remain stable.


`readiness::validate_hpc_submission_ready` consumes `ReadinessContext` and a report path. The context borrows a `LocalReadinessBackend` and explicit input path declarations. Provider traits are grouped by independent responsibilities under `readiness::ports`. A failed assessment still publishes the full validated report and returns an error. Provider projections do not claim fresh scientific execution or tool equivalence.

`dry_run::{render_hpc_dry_run_ready, validate_hpc_dry_run_ready_path}` consumes `DryRunContext` with explicit repository root, canonical provider paths, nine typed evidence operations and output policy. Output admission precedes acquisition. Validation parses and semantically validates the persisted report before reacquisition and exact byte comparison. Failed assessments still publish all nine diagnostics; they do not certify remote readiness.

`candidate` consumes `CandidateContext` and an explicit output path. Provider operations return the existing asset, execution, job and resource contracts plus a nonserialized two-path result observation. Canonical model validators admit asset/execution inputs and the output report. Invalid durations and overflowing byte totals reject before result-path observation/publication. Replay reacquires inputs; no submission operation is exposed.
