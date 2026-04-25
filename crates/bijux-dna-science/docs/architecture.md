# Architecture

`bijux-dna-science` separates four concerns:

- `domain/`: typed records and compiled row models
- `schema/`: schema version constants and authored-surface rules
- `compile/`: loading, validation, reference resolution, and slice compilers
- `render/`: deterministic TSV and JSON output formatting

The crate reads authored specs from `science/specs/**`, compiles deterministic outputs under
`science/generated/**`, and optionally freezes science release bundles under
`artifacts/science-releases/**`.
