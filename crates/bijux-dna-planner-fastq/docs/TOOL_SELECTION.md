# TOOL_SELECTION

## Authority
FASTQ planner tool admission and defaults come from `domain/fastq/execution_support.yaml`
through `bijux-dna-domain-fastq`. The planner must not derive FASTQ execution support from
registry files or environment-variable toggles.

## Decision criteria
- Quality: accuracy and robustness of outputs.
- Speed: runtime and resource usage.
- Compatibility: output artifacts match stage contracts.
- Alternatives: documented when excluded (e.g., missing metrics, non-deterministic behavior).
