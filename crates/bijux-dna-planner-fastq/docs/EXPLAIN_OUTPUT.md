# Explain Output

Explain output is carried in plan decision reasons and snapshot payloads. It must make tool selection and defaults visible without requiring runtime execution.

## Required Concepts
- selected tool IDs
- defaults diff
- deterministic selection reason
- contract hash when available
- effective params used to build the command spec

## Example
```json
{
  "selected_tools": ["fastp"],
  "defaults_diff": {},
  "reasons": ["fastp provides trim+filter in one step"],
  "contract_hashes": {"fastq.trim_reads": "sha256:..."}
}
```

## Enforcement
- `tests/contracts/explain/explainability.rs`
- `tests/contracts/explain/docs_explainability.rs`
- snapshots under `tests/snapshots/`
