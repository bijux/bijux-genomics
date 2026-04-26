# Invariants

Core invariants are the behaviors downstream crates may rely on without
revalidating contract shape themselves. When an invariant changes, update this
file and the closest test in the same change set.

## Contract Invariants

- Canonical JSON object keys are stable:
  `tests/contracts/surface/canonicalization.rs::canonicalize_json_value_sorts_keys`.
- Parameter JSON canonicalization is deterministic for numeric JSON values:
  `tests/contracts/surface/canonicalization.rs::parameters_json_canonicalization_normalizes_numbers`.
- Metrics schemas resolve for known stages:
  `tests/contracts/surface/canonicalization.rs::metrics_schema_resolves_stage`.
- Execution graphs reject cycles:
  `tests/contracts/execution/execution_graph_validate.rs::validate_rejects_cycles`.
- Execution graph serialization stays free of stage-plan and plugin payloads:
  `tests/boundaries/execution_graph_purity.rs::execution_graph_serialization_is_stage_plan_free`.

## ExecutionGraph
### Valid (acyclic)
```json
{
  "schema_version": "bijux.execution_graph.v1",
  "contract_version": {"major": 1, "minor": 0},
  "pipeline_id": "fastq.default.v1",
  "planner_version": "planner.v1",
  "policy": {"mode": "strict"},
  "steps": [{"step_id": "step.a", "stage_id": "stage.a", "command": {"template": []}, "image": {"image": "x", "digest": null}, "resources": {"runtime": "local", "mem_gb": 1, "tmp_gb": 1, "threads": 1}, "io": {"inputs": [], "outputs": []}, "out_dir": "out"}],
  "edges": []
}
```
### Invalid Cycle

```json
{"edges": [{"from": "step.a", "to": "step.a"}]}
```

Enforced by
`tests/contracts/execution/execution_graph_validate.rs::validate_rejects_cycles`.

## Stage plan purity

Execution graphs must not serialize stage-plan or plugin types.
Enforced by
`tests/boundaries/execution_graph_purity.rs::execution_graph_serialization_is_stage_plan_free`.
