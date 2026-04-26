# bijux-dna-domain-bam Scope

## Belongs here
- BAM stage ids and stage ordering.
- BAM params and deterministic default presets.
- BAM metric schemas, parser helpers, and invariant semantics.
- BAM stage artifact policies and serialized stage contract JSON.
- Small source-controlled fixtures used to prove those contracts.

## Does not belong here
- Tool selection or planner orchestration.
- Process execution, container/runtime behavior, or environment inspection.
- Network access or generated config writes.
- CLI command ownership.

The repository-level policy lives at `README.md`.
