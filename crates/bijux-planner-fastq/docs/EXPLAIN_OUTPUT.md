# EXPLAIN_OUTPUT

## Fields
- selected_tools
- defaults_diff
- reasons
- contract_hashes

## Canonical example
```json
{
  "selected_tools": ["fastp"],
  "defaults_diff": {},
  "reasons": ["fastp provides trim+filter in one step"],
  "contract_hashes": {"fastq.trim": "sha256:..."}
}
```

See `tests/explainability.rs` for enforcement.
