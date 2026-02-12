# PUBLIC_API

## Stable surface
- `client` for ENA filereport query construction and parsing.
- `download` for deterministic download task planning and execution.
- `model` for typed ENA row/model normalization.

## Compatibility notes
- CLI flags and JSON output are versioned by the binary crate.
- Internal structs may evolve; serialized artifacts must remain backward-compatible when persisted.
