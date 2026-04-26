# Architecture

`bijux-dna-planner-fastq` is a planner crate. The architecture keeps public surface, pipeline planning, stage composition, tool selection, and tool-specific command spec construction in separate modules.

## Layout
- `src/lib.rs` exposes `surface`, `stage_api`, and `tool_adapters`.
- `src/surface.rs` centralizes root-level reexports and constants.
- `src/stage_api.rs` exposes stage-level compatibility helpers and governance views.
- `src/planner/` owns graph planning, route expansion, benchmark fan-out, graph policy, and planner-local support types.
- `src/compose/` owns input resolution, stage parameters, route lineage, report-QC input
  collection, and stage binding composition.
- `src/preprocess/` owns preprocess policy and pipeline choice.
- `src/selection/` owns tool allowlisting, override merging, and selection helpers.
- `src/tool_adapters/` owns stage-specific command spec construction.
- `src/qc_contract.rs` owns governed QC contributor relationships.
- `src/report_stage.rs` owns the report aggregation graph step.

## Stage Families
- `tool_adapters/stages/pre/` covers validation, read profiling, adapter detection, and reference indexing.
- `tool_adapters/stages/qc/` covers QC reports, taxonomy screening, and rRNA depletion.
- `tool_adapters/stages/transform/` covers trim, merge, filter, deduplicate, correction, UMI, and depletion transforms.
- `tool_adapters/stages/amplicon/` covers primer normalization, chimera removal, ASV/OTU processing, and abundance normalization.

## Design Rules
- Keep root files as facades or stable subsystem entrypoints.
- Keep domain truth in `bijux-dna-domain-fastq`; do not duplicate stage/tool matrices in planner docs.
- Keep command templates in `tool_adapters/`; do not hide runtime parsing in planner glue.
- Keep benchmark fan-out graph construction in `planner/`; do not mix it into stage adapters.
- Update this map and the architecture tree test when the layout changes intentionally.
