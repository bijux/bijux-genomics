# Architecture

`bijux-dna-planner-bam` turns BAM pipeline profiles and stage contracts into deterministic stage plans and execution graphs. It emits command specs for downstream execution, but does not run them.

## Source Layout
- `src/lib.rs` owns public planning entrypoints and pipeline helpers.
- `src/api.rs` owns request/config structs and the `stage_api` compatibility surface.
- `src/profile_catalog.rs` maps supported BAM pipeline IDs to upstream pipeline profiles and ordered stage lists.
- `src/selection/` reads the repository tool registry and resolves allowed/default tools.
- `src/stage_dispatch/` dispatches BAM stages to the correct planning family.
- `src/tool_adapters/` builds `StagePlanV1` command specs for concrete BAM tools.
- `src/stages/` exposes stage registry projection from tool registry and BAM domain contracts.
- `src/execution_graph.rs` projects stage plans into `ExecutionGraph`.
- `src/params.rs`, `src/tool_policy.rs`, `src/stage_activation.rs`, and `src/report_stage.rs` own focused validation and helper behavior.

## Data Flow
1. A caller provides a `StagePlanRequest` or `BamPipelineInputs`.
2. Pipeline helpers load a BAM profile and ordered BAM stages from `bijux-dna-pipelines`.
3. Tool selection resolves allowed/default tools from repository configuration and domain contracts.
4. Stage dispatch validates required inputs and typed params for the selected BAM stage.
5. Tool adapters build deterministic `StagePlanV1` command specs.
6. `execution_graph::from_stage_plans` derives deterministic graph steps and edges.

## Supported Pipeline Helpers
- `plan_bam_to_bam__adna_shotgun__v1`
- `plan_bam_to_bam__adna_capture__v1`
- `pipeline_id_catalog`

## Planned Stage Groups
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
- `bam.bias_mitigation`
- `bam.haplogroups`
- `bam.genotyping`
- `bam.kinship`

Downstream analysis stages require the `bam_downstream` feature for concrete planning.

## Tool Selection Rules
- Prefer configured primary tools from the repository tool registry.
- Fall back to BAM domain tool contracts when repository configuration is unavailable.
- Sort candidates deterministically.
- Reject silent planning when no compatible tool candidate exists.

## Adding a Tool or Stage Adapter
- Add the adapter in the matching `src/tool_adapters/` family.
- Wire stage dispatch when the stage is newly supported.
- Update the repository tool registry when selection candidates change.
- Update `docs/COMMANDS.md` when a planned stage command becomes newly supported.
- Add or refresh plan, graph, explain, and command snapshots when output changes intentionally.

## Change Rules
- Public API changes require `docs/PUBLIC_API.md` and boundary tests to change together.
- Plan JSON, graph topology, command templates, explain payloads, and default tool selection changes require snapshot review.
- Execution behavior belongs downstream and must not be added here.
