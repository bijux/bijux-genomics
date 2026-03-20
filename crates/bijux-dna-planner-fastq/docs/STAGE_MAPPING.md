# STAGE_MAPPING

This document is intentionally not a manual stage-to-tool matrix.

Authoritative FASTQ stage and tool truth lives in:

- `domain/fastq/index.yaml` for declared stage and tool families
- `domain/fastq/execution_support.yaml` for governed execution closure and maturity
- `domain/fastq/stages/*.yaml` for governed stage contracts
- `domain/fastq/tools/*.yaml` for tool execution contracts and `stage_contracts`
- `crates/bijux-dna-planner-fastq/src/tool_adapters/fastq.rs` and `stage_api` for the closed planner surface derived from those manifests

Generated configs and runtime registries must publish only the closed FASTQ execution surface.
If a binding is declared, planned, or out of scope, keep that truth in the manifests and tests rather than copying it into a Markdown table.

Use this document only to explain planner interpretation rules:

- FASTQ planner admission follows manifest-governed stage compatibility plus execution-support closure.
- Runtime registry publication follows the closed governed surface, not every declared or planned binding.
- Bench cohort selection may include only bindings whose normalization and comparison contracts are explicitly governed.

Declared-only FASTQ stages:

- `fastq.infer_asvs` remains defined in the domain with planned `dada2` intent, but it stays outside the governed runtime registry and governed metrics surface until execution support is closed.
