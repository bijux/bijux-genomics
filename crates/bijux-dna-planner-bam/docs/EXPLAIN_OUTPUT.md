# Explain Output

Explain output makes planner decisions auditable. It must describe what was selected and why without requiring command execution.

## Guarantees
- Selected tools are emitted in stable order.
- `defaults_diff` is present in plan reason details.
- Tool-selection reasons use stable reason kinds and messages.
- Stage contract hashes are included when available.
- Explain snapshots cover ancient-DNA BAM stages with high review value.

## Canonical Shape
```json
{
  "selected_tools": ["bwa", "samtools"],
  "defaults_diff": {},
  "reasons": ["bwa for alignment", "samtools for sort/index"],
  "contract_hashes": {"bam.align": "sha256:..."}
}
```

See `tests/contracts/explain/explainability.rs` and `tests/contracts/graph/graph_snapshots.rs` for enforcement.
