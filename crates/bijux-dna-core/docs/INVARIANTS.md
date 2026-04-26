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
- Run identity is independent of input hash order and duplicates:
  `tests/contracts/identity/hashing_identity.rs::run_id_is_order_independent_for_input_hashes`.
- Input fingerprints are independent of input hash order and duplicates:
  `tests/contracts/identity/hashing_identity.rs::input_fingerprint_is_order_independent_and_deduped`.
- Execution graphs reject cycles:
  `tests/contracts/execution/execution_graph_validate.rs::validate_rejects_cycles`.
- Execution graph serialization stays free of stage-plan and plugin payloads:
  `tests/boundaries/execution_graph_purity.rs::execution_graph_serialization_is_stage_plan_free`.
- Identifier validators reject invalid pipeline, stage, tool, artifact, and profile
  shapes:
  `tests/contracts/surface/metrics_ids_selection_contracts.rs::ids_validation_covers_success_and_failure_paths`.
- Input assessment discovers FASTQ files in stable order and records paired,
  single-end, duplicate, orphan, and persisted assessment behavior:
  `tests/semantics/input_assessment.rs`.

## ExecutionGraph
### Valid (acyclic)
```json
{
  "schema_version": "bijux.execution_graph.v1",
  "contract_version": {"major": 1, "minor": 0},
  "pipeline_id": "fastq-to-fastq__default__v1",
  "planner_version": "planner.v1",
  "policy": "PreferAccuracy",
  "deterministic_scheduler": true,
  "retry_policy": {"max_attempts": 1, "retry_on_exit_codes": []},
  "steps": [{
    "step_id": "step.a",
    "stage_id": "fastq.validate_reads",
    "command": {"template": ["fastqvalidator", "reads.fastq.gz"]},
    "image": {"image": "fastqvalidator", "digest": null},
    "resources": {"runtime": "local", "mem_gb": 1, "tmp_gb": 1, "threads": 1},
    "io": {
      "inputs": [{"name": "reads_in", "path": "reads.fastq.gz", "role": "reads", "optional": false}],
      "outputs": [{"name": "report", "path": "report.json", "role": "report_json", "optional": false}]
    },
    "out_dir": "out",
    "aux_images": {},
    "expected_artifact_ids": ["report"],
    "metrics_schema_ids": []
  }],
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
