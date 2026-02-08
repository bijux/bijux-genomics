# Boundary Map

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-core, bijux-engine, bijux-runtime, bijux-runner, bijux-api

## What
Points to the canonical boundary diagram and dependency rules.

## Why
Avoids boundary duplication across documents.

## Non-goals
- Restating dependency rules.

## Contracts
Enforced by:
- `docs/10-architecture/BOUNDARY_DIAGRAM.md`
- `docs/10-architecture/DEPENDENCY_RULES.md`
- `crates/bijux-policies/tests/deps/dependency_boundaries.rs`
- `crates/bijux-policies/tests/deps/effect_boundary_map.rs`

## Executable Boundary Map
```boundaries
bijux-dna-cli: bijux-api bijux-core bijux-infra bijux-policies
bijux-analyze: bijux-benchmark bijux-core bijux-domain-bam bijux-domain-fastq bijux-infra bijux-pipelines bijux-planner-bam bijux-planner-fastq bijux-policies bijux-runtime bijux-testkit
bijux-api: bijux-analyze bijux-benchmark bijux-core bijux-domain-bam bijux-domain-fastq bijux-engine bijux-environment bijux-environment-qa bijux-infra bijux-pipelines bijux-planner-bam bijux-planner-fastq bijux-policies bijux-runner bijux-runtime bijux-stage-contract bijux-testkit
bijux-benchmark: bijux-analyze bijux-benchmark-model bijux-core bijux-domain-bam bijux-domain-fastq bijux-infra bijux-policies bijux-runtime bijux-testkit
bijux-benchmark-model: bijux-analyze bijux-core bijux-policies bijux-testkit
bijux-core: bijux-infra bijux-policies bijux-testkit
bijux-domain-bam: bijux-core bijux-policies bijux-testkit
bijux-domain-fastq: bijux-core bijux-infra bijux-policies bijux-testkit
bijux-engine: bijux-core bijux-infra bijux-policies bijux-runtime bijux-testkit
bijux-environment: bijux-core bijux-infra bijux-policies bijux-runtime bijux-testkit
bijux-environment-qa: bijux-analyze bijux-core bijux-domain-fastq bijux-environment bijux-infra bijux-policies bijux-runtime bijux-testkit
bijux-infra: bijux-policies bijux-testkit
bijux-pipelines: bijux-core bijux-domain-bam bijux-domain-fastq bijux-policies bijux-testkit
bijux-planner-bam: bijux-core bijux-domain-bam bijux-infra bijux-pipelines bijux-policies bijux-stage-contract bijux-stages-bam bijux-testkit
bijux-planner-fastq: bijux-core bijux-domain-bam bijux-domain-fastq bijux-infra bijux-pipelines bijux-policies bijux-stage-contract bijux-stages-fastq bijux-testkit
bijux-policies: bijux-testkit
bijux-runner: bijux-core bijux-environment bijux-infra bijux-policies bijux-runtime
bijux-runtime: bijux-core bijux-infra bijux-policies bijux-testkit
bijux-stage-contract: bijux-core bijux-policies bijux-testkit
bijux-stages-bam: bijux-core bijux-domain-bam bijux-infra bijux-policies bijux-stage-contract bijux-testkit
bijux-stages-fastq: bijux-core bijux-domain-fastq bijux-infra bijux-policies bijux-runtime bijux-stage-contract bijux-testkit
bijux-testkit: bijux-policies
```

## Examples
See `BOUNDARY_DIAGRAM.md` for the canonical diagram.

## Failure modes
Boundary violations fail CI dependency/effect policies.
