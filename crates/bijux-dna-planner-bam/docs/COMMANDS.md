# Commands

`bijux-dna-planner-bam` owns no runtime CLI commands. It does own deterministic planned command specs for BAM stages; runners may execute those specs downstream, but this crate must not execute them.

## Runtime Commands
None.

## Planned BAM Stage Commands
The planner can produce command specs for these BAM stage IDs through `plan_stage` and the pipeline helpers:

### Pre-Alignment and Filtering
- `bam.align`
- `bam.validate`
- `bam.qc_pre`
- `bam.mapping_summary`
- `bam.filter`
- `bam.mapq_filter`
- `bam.length_filter`
- `bam.overlap_correction`

### Post-Alignment QC
- `bam.markdup`
- `bam.duplication_metrics`
- `bam.complexity`
- `bam.coverage`
- `bam.insert_size`
- `bam.gc_bias`
- `bam.endogenous_content`
- `bam.recalibration`

### Ancient-DNA Analysis
- `bam.damage`
- `bam.authenticity`
- `bam.contamination`
- `bam.sex`

### Downstream Analysis
These require the `bam_downstream` feature for concrete planning:

- `bam.bias_mitigation`
- `bam.haplogroups`
- `bam.genotyping`
- `bam.kinship`

## Boundaries
- No `src/bin/` entrypoints.
- No CLI parsing or command routing.
- No process spawning.
- No network access.
- No product execution.

## Validation
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-bam --no-default-features --test boundaries command_inventory_documents_planned_bam_stage_commands
```
