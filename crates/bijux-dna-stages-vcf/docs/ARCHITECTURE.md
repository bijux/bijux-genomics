# Architecture

This architecture note is intentionally brief. It records the stable VCF crate map and points deeper behavior to the focused docs that already carry the contract.

## Layout
- `lib.rs` exposes the crate surface.
- `engine/` owns request models, stage dispatch, entrypoints, and reporting support.
- `pipeline/` owns execution families for calling, qc, orchestration, population-panel work, and imputation.
- `pipeline_sections/` holds shared execution support that still serves multiple pipeline families.
- `stage_specs.rs`, `metrics.rs`, `invariants.rs`, `path_contract.rs`, `repo_root.rs`, `vcf_io.rs`, and `wrappers.rs` stay as narrow top-level support modules.

## Change rules
- Add new top-level files only for distinct enduring concerns.
- Prefer explicit submodules over include-based wiring.
- Remove superseded duplicates in the same change that replaces them.
- Update this map and the boundary tree contract together when the layout changes intentionally.

## Pointers
- `INDEX.md` for the documentation map.
- `FEATURES.md`, `SCOPE.md`, and `EFFECTS.md` for crate behavior.
- `TESTS.md` for verification expectations.
