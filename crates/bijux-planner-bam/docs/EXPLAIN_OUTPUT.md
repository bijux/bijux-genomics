# EXPLAIN_OUTPUT

## Canonical example
```json
{
  "selected_tools": ["bwa", "samtools"],
  "defaults_diff": {},
  "reasons": ["bwa for alignment", "samtools for sort/index"],
  "contract_hashes": {"bam.align": "sha256:..."}
}
```

See `tests/graph_snapshots.rs` for canonical graphs.
