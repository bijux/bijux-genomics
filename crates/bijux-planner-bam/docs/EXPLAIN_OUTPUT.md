# EXPLAIN_OUTPUT

## Guarantee
Explain output includes:
- `selected_tools` in stable order
- `defaults_diff` (profile vs pipeline)
- `reasons` for tool selection
- `contract_hashes` for each stage

## Canonical example
```json
{
  "selected_tools": ["bwa", "samtools"],
  "defaults_diff": {},
  "reasons": ["bwa for alignment", "samtools for sort/index"],
  "contract_hashes": {"bam.align": "sha256:..."}
}
```

See `tests/contracts/explain/explainability.rs` and `tests/contracts/graph/graph_snapshots.rs` for enforcement.
