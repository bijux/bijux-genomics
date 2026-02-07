# Crate Map

| Crate | Role | SSOT ownership | Purity guarantees | Key types |
| --- | --- | --- | --- | --- |
| bijux-core | Contract bible | IDs + canonicalization | No effects | `ExecutionGraph`, `RunManifest` |
| bijux-engine | Orchestrator | None | No execution | `Engine` |
| bijux-runtime | Recording + layout | Run layout | Effect‑free except run layout | `RunLayout` |
| bijux-runner | Execution backends | None | Allowed effects | `Runner` |
| bijux-api | User orchestration | None | No direct execution | `PlanRequest` |
| bijux-stages-* | Stage specs + observers | Stage truth | No effects | `StageSpec` |
| bijux-stage-contract | Planning contract | Plan types | No effects | `StagePlanV1` |
| bijux-planner-* | Tool selection | Selection logic | No effects | `Planner` |
| bijux-pipelines | Profiles/presets | Pipeline IDs | No effects | `PipelineProfile` |
| bijux-analyze | Reporting | None | No execution | `Report` |
| bijux-benchmark | Comparisons | None | No execution | `BenchmarkSummary` |
| bijux-domain-* | Params + metrics | Domain truth | No effects | metrics/params types |
| bijux-infra | Utilities | None | No effects | helpers |
| bijux-environment | Environment specs | Environment truth | No execution | `EnvironmentSpec` |
| bijux-environment-qa | QA harness | None | Allowed effects | QA fixtures |
| bijux-cli | UX boundary | None | No direct execution | CLI commands |
