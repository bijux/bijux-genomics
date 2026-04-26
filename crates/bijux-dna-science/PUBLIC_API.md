# bijux-dna-science Public API

Public modules exported from `src/lib.rs`:
- app
- cli
- compile
- domain
- errors
- io
- release
- render
- schema

Stable entrypoints:
- `app::validate_workspace` validates authored specs.
- `app::build_workspace` refreshes governed generated outputs.
- `app::trace_workspace` returns filtered FASTQ environment evidence rows.
- `app::release_workspace` writes immutable science release bundles.
- `compile::compile_workspace` returns the compiled science model without writing outputs.
