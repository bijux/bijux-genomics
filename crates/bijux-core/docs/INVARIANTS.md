# INVARIANTS

## Contract invariants (must always hold)
- Canonical JSON sorting is stable (`tests/contract/canonicalization.rs::canonicalize_json_value_sorts_keys`).
- Parameters JSON canonicalization normalizes numbers (`tests/contract/canonicalization.rs::parameters_json_canonicalization_normalizes_numbers`).
- Metrics schemas resolve for known stages (`tests/contract/canonicalization.rs::metrics_schema_resolves_stage`).
- Execution graphs are acyclic (`tests/contract/execution_graph_validate.rs::validate_rejects_cycles`).
- Execution graph serialization is stage-plan free (`tests/contract/execution_graph_purity.rs::execution_graph_serialization_is_stage_plan_free`).

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
### Invalid (cycle)
```json
{"edges": [{"from": "step.a", "to": "step.a"}]}
```
Enforced by `tests/contract/execution_graph_validate.rs::validate_rejects_cycles`.

## Stage plan purity
Execution graphs must not serialize stage-plan or plugin types.
Enforced by `tests/contract/execution_graph_purity.rs::execution_graph_serialization_is_stage_plan_free`.
