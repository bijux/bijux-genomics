# DATA_MODEL

## Schema Map
- facts: run inputs, tool identity, step metadata
- metrics: per-stage metrics envelopes
- report outputs: report.json, report.html, summary.tsv

## Stability Guarantees
- Additive fields are backward compatible.
- Breaking changes require a major version bump.
