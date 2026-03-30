# Architecture

## Goals
- Keep the crate root thin and stable.
- Separate pipeline execution concerns by family instead of assembling them through flat include chains.
- Keep engine request, orchestration, and reporting support explicit.
- Preserve a small amount of legacy layout only where it still owns live code.

## Source tree

```text
src/
├── engine/
│   ├── entrypoints.rs
│   ├── mod.rs
│   ├── reporting.rs
│   ├── request.rs
│   ├── stage_runner.rs
│   └── wrappers.rs
├── invariants.rs
├── lib.rs
├── metrics.rs
├── path_contract.rs
├── pipeline/
│   ├── calling/
│   │   ├── damage_and_propagation.rs
│   │   ├── helpers.rs
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── imputation/
│   │   ├── execution_engine.rs
│   │   ├── execution_outputs.rs
│   │   ├── impl.rs
│   │   ├── mod.rs
│   │   ├── postprocess.rs
│   │   ├── stage_logic.rs
│   │   └── workflow.rs
│   ├── mod.rs
│   ├── orchestration/
│   │   ├── mod.rs
│   │   └── tail.rs
│   ├── population_panel/
│   │   ├── analysis_and_panel.rs
│   │   ├── helpers.rs
│   │   └── mod.rs
│   └── qc/
│       ├── mod.rs
│       └── stage_params.rs
├── pipeline_sections/
│   └── execution/
│       └── chunking_and_resume.rs
├── repo_root.rs
├── stage_specs.rs
├── vcf_io.rs
└── wrappers.rs
```

## Responsibilities
- `engine/`: public request/result types, stage dispatch, pipeline entrypoints, and report/explain writing.
- `pipeline/calling/`: call, filter, GL propagation, and damage-filter execution.
- `pipeline/qc/`: QC and shared stage parameter models for downstream VCF stages.
- `pipeline/population_panel/`: population analysis and reference-panel preparation helpers.
- `pipeline/orchestration/`: phasing and orchestration control flow.
- `pipeline/imputation/`: imputation workflow and postprocess execution artifacts.
- `pipeline_sections/execution/chunking_and_resume.rs`: chunk-planning support still shared directly by the pipeline facade.

## Change rules
- Add new top-level files only when they own a distinct enduring concern.
- Prefer explicit submodules over include-based wiring.
- Remove superseded duplicate source files in the same change that replaces them.
- Update this document and the tree contract together when the layout changes intentionally.

## Failure mode
- Unexpected files or misplaced modules should fail the boundary architecture test.
