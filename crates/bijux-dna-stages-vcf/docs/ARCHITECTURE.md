# Architecture

This crate is the current VCF stage execution implementation. It is not only a
metadata crate: it validates VCF inputs, prepares artifacts, runs typed stage
helpers, records manifests, and parses stage metrics.

## Layout

- `lib.rs` exposes public modules and the implemented stage registry helper.
- `engine/` owns dispatch request/result models, stage execution dispatch,
  refusal mapping, manifests, runtime explanation artifacts, and wrapper checks.
- `pipeline/` owns typed VCF stage families: calling, QC, orchestration,
  population-panel work, and imputation.
- `pipeline_sections/` holds shared execution support that still serves multiple
  pipeline families.
- `stage_specs.rs` owns the VCF stage catalog and support metadata.
- `metrics.rs`, `invariants.rs`, `path_contract.rs`, `repo_root.rs`,
  `vcf_io.rs`, and `wrappers.rs` stay as focused support modules.

## Change rules

- Add new top-level files only for distinct enduring concerns.
- Prefer explicit submodules over include-based wiring when moving shared
  pipeline-section code.
- Remove superseded duplicates in the same change that replaces them.
- Update this map and the boundary tree contract together when the layout changes intentionally.

## Pointers

- `COMMANDS.md` for the managed operation inventory.
- `BOUNDARY.md` for ownership boundaries.
- `DEPENDENCIES.md` for dependency graph shape.
- `EFFECTS.md` for allowed and forbidden effects.
- `TESTS.md` for verification expectations.
