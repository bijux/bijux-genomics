# INVARIANTS

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
Enforced by `tests/execution_graph_validate.rs`.

## Unique IDs
Invalid:
```json
{"steps": [{"step_id": "dup"}, {"step_id": "dup"}]}
```
Enforced by `tests/execution_graph_validate.rs`.

## Artifact resolvability
Invalid:
```json
{"steps": [{"expected_artifact_ids": ["missing"]}]}
```
Enforced by `tests/execution_graph_validate.rs`.
