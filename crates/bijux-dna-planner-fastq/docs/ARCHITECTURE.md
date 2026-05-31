# Architecture

`bijux-dna-planner-fastq` is a planner crate. The architecture keeps public surface, pipeline planning, stage composition, tool selection, and tool-specific command spec construction in separate modules.

## Layout
- `src/lib.rs` exposes `surface`, `stage_api`, and `tool_adapters`.
- `src/surface.rs` centralizes root-level reexports and constants.
- `src/stage_api.rs` exposes stage-level compatibility helpers and governance views.
- `src/planner/` owns graph planning, route expansion, benchmark fan-out, graph policy, and planner-local support types.
- `src/planner/local_readiness.rs` owns governed local-ready FASTQ stage-plan construction from repository config.
- `src/compose/` owns input resolution, stage parameters, route lineage, report-QC input
  collection, and stage binding composition.
- `src/preprocess/` owns preprocess policy and pipeline choice.
- `src/selection/` owns tool allowlisting, override merging, and selection helpers.
- `src/tool_adapters/` owns stage-specific command spec construction.
- `src/qc_contract.rs` owns governed QC contributor relationships.
- `src/report_stage.rs` owns the report aggregation graph step.

## Stage Families
```text
src/tool_adapters/stages/
  catalog.rs                    stage adapter registry surface
  pre/
    detect_adapters.rs          adapter-evidence planning
    index_reference.rs          reference-index planning
    plan_preprocess.rs          preprocess route assembly
    preprocess.rs               preprocess stage plan helpers
    profile_overrepresented_sequences.rs
    profile_read_lengths.rs
    validate_reads.rs
  qc/
    deplete_rrna.rs             rRNA depletion planning
    profile_reads.rs            read profiling planning
    report_qc.rs                QC report aggregation planning
    screen_taxonomy.rs          taxonomy screening planning
  transform/
    correct_errors.rs           read correction planning
    deplete_host.rs             host depletion planning
    deplete_reference_contaminants.rs
    extract_umis.rs
    filter_low_complexity.rs
    filter_reads.rs
    merge_pairs.rs
    remove_duplicates.rs
    trim_polyg_tails.rs
    trim_reads/                 trim planning, config, and reporting helpers
    trim_terminal_damage.rs
  amplicon/
    cluster_otus.rs
    infer_asvs.rs
    normalize_abundance.rs
    normalize_primers.rs
    remove_chimeras.rs
```

## Design Rules
- Keep root files as facades or stable subsystem entrypoints.
- Keep domain truth in `bijux-dna-domain-fastq`; do not duplicate stage/tool matrices in planner docs.
- Keep command templates in `tool_adapters/`; do not hide runtime parsing in planner glue.
- Keep benchmark fan-out graph construction in `planner/`; do not mix it into stage adapters.
- Update this map and the architecture tree test when the layout changes intentionally.
