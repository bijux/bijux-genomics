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
- `plugin/` adapts every VCF domain stage to the shared `StagePlugin`
  invocation and normalized-output contract without selecting tools.
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

`vcf_io/indexing.rs` owns the shared VCF-to-BGZF/index encoder and paired publication. Inputs up to 1 MiB use bounded encoding and retain existing regular files only when both newly encoded payloads match exactly. Larger inputs stream through the same encoder. Missing, changed, corrupt or symlink outputs are replaced through the existing staged publication path.

`pipeline/imputation/chunk_regions.rs` projects imputation identifiers from acquired
contig observations. Nonzero windows use `plan_regions_deterministic` for overlap
admission and endpoint arithmetic, retaining the first declared contig length,
selection filters and final lexical identifier order. `None` and `Some(0)` retain
whole-contig planning. Invalid overlap is refused even for empty or fully filtered
fixed-window selections. Planning remains at the existing execution-output call
position, after earlier acquisition, validation and BGZF/QC publication; this
boundary does not promise rollback of those earlier outputs.
