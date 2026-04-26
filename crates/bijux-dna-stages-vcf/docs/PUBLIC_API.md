# Public API

The crate root exports the VCF stage execution modules from `src/lib.rs`.

## Public Modules

- `engine`
- `invariants`
- `metrics`
- `path_contract`
- `pipeline`
- `stage_specs`
- `vcf_io`
- `wrappers`

## Stable Root Exports

- `implemented_stages`

## Compatibility Rules

- Removing or renaming a public module is breaking.
- Changing stage output paths, manifest shapes, refusal codes, metrics schemas,
  or wrapper-check semantics is breaking unless versioned explicitly.
- Adding a managed operation requires an entry in `docs/COMMANDS.md`.
- Adding a stage requires stage catalog, command inventory, docs, and contract
  test coverage.

## Internal Modules

- `repo_root` is private support for workspace configuration lookup.
- Private submodules under `engine`, `pipeline`, and `pipeline_sections` should
  stay private unless a caller has a durable contract need.
