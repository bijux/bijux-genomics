# bijux-dna-science Public API

## Public Modules

The crate root exports these modules from `src/lib.rs`:

- `app`
- `cli`
- `compile`
- `domain`
- `errors`
- `io`
- `release`
- `render`
- `schema`

## Stable Entrypoints

- `app::run` dispatches parsed CLI commands.
- `app::validate_workspace` validates authored science specs.
- `app::build_workspace` refreshes governed generated science outputs.
- `app::trace_workspace` returns filtered FASTQ environment evidence rows.
- `app::release_workspace` writes immutable science release bundles.
- `compile::load_specs` loads and validates authored science specs.
- `compile::compile_workspace` compiles authored specs into deterministic science rows.
- `compile::compile_loaded` compiles already loaded specs.
- `release::cut_release` writes a science release bundle.

## Data Surface

`domain` owns typed science identifiers, authored spec structs, compiled row structs,
`ScienceIndex`, `FastqClosureSummary`, `FastqEvidenceSummary`, and `CompiledScience`.
`render` owns TSV and JSON rendering for the compiled rows. `schema` owns the accepted
authored spec schema version constants.

`ScienceIndex` is the stable JSON index surface for generated science output. Its FASTQ
summary fields are intended for operator entrypoints and governance checks that need
rolled-up state before drilling into individual TSV rows.
