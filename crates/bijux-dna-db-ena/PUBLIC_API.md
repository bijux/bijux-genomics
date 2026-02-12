# PUBLIC_API

## Stable surface
- client
  - ENA filereport query construction and parsing.
- download
  - Deterministic download task planning and execution.
- model
  - Typed ENA row/model normalization.

## Compatibility notes
- CLI flags and JSON output are versioned by the binary crate.
- Internal structs may evolve; serialized artifacts must remain backward-compatible when persisted.
