# Contract

## Compatibility matrix

| Contract | Planner | Runtime | Analyze |
| --- | --- | --- | --- |
| v1 | supported | supported | supported |

## Breaking change definition

Breaking change = major bump. Examples:

- Removing fields
- Renaming fields
- Changing semantics

Tests under `tests/versioning/*` enforce that breaking changes require a major bump.

Non-breaking changes may use a minor bump when callers can safely ignore them:

- Adding an optional field.
- Adding a new enum variant with safe default handling.
- Adding a new snapshot fixture without changing existing fixture semantics.

## Terminology

- **Plan**: a planned set of steps (this crate).
- **Run**: an executed plan with runtime artifacts (runtime/runner crates).
- **Execution plan**: the serialized plan JSON defined by this crate.
- **Canonical stage contract**: the typed stage/backend agreement that binds
  semantic stage ID, backend tool ID, artifact roles, capability claims,
  report contracts, refusal codes, and operating mode before execution.

## No execution detail
This crate defines planning contracts only; execution belongs in core/runtime.
For execution manifests and run contracts, `bijux-dna-core` is the authority.

## Example

`docs/EXAMPLE_PLAN.json` is the raw fixture. Field notes:

- `contract_version`: version of the contract; breaking changes require a major
  bump.
- `schema_version`: schema name for the serialized plan.
- `plan_id`: unique ID for this plan instance.
- `pipeline_id`: registry ID owned by `bijux-dna-pipelines`.
- `planner_id`: planner that produced the plan.
- `steps`: ordered list of planned stage steps.
- `step_id`: unique step identifier, often equal to `stage_id`.
- `stage_id`: canonical stage identifier from domain contracts.
- `tool_id` and `tool_version`: selected tool metadata for planning only.
- `inputs` and `outputs`: artifact IDs plus relative paths for each step.
- `params`: explicit parameters for the step; hidden defaults are not allowed.

## Canonical Stage Contract Fixtures

The executable stage-contract examples live under `tests/fixtures/docs/` and are
validated by the schema suite:

- `tests/fixtures/docs/fastq_trim_stage_contract.json`
- `tests/fixtures/docs/bam_align_stage_contract.json`
- `tests/fixtures/docs/vcf_filter_stage_contract.json`

Each fixture demonstrates the same canonical contract shape with different
semantic stages and backend tools:

- FASTQ trimming: semantic stage `fastq.trim_reads` distinct from backend
  `fastp`.
- BAM alignment: semantic stage `bam.align` distinct from backend `bwa`.
- VCF filtering: semantic stage `vcf.filter` distinct from backend `bcftools`.

These fixtures also exercise typed artifact roles, deterministic parameter
aliases/defaults, refusal codes, report-contract typing, and
simulation/advisory/enforced operating semantics.

## Contract Promise

This crate owns planning contracts only. It does not own execution, IO layout,
tool selection policy, command routing, or runtime mutation.
