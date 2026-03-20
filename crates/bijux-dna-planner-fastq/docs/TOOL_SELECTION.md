# TOOL_SELECTION

## Authority
FASTQ planner tool admission and defaults come from `domain/fastq/execution_support.yaml`
through `bijux-dna-domain-fastq`. The planner must not derive FASTQ execution support from
registry files or environment-variable toggles.

## Runtime boundary
`configs/ci/stages/stages.toml` and `configs/ci/registry/tool_registry.toml` publish only the
closed FASTQ execution surface. Domain manifests may still describe declared-only stages and
planned tool bindings, but those entries must stay out of the governed runtime catalog until the
stage contract, planner adapter, and shipped runtime support are all closed together.

## Decision criteria
- Quality: accuracy and robustness of outputs.
- Speed: runtime and resource usage.
- Compatibility: output artifacts match stage contracts.
- Alternatives: documented when excluded (e.g., missing metrics, non-deterministic behavior).
