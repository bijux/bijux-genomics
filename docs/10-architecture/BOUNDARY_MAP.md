# Boundary Map

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-dna-core, bijux-dna-engine, bijux-dna-runtime, bijux-dna-runner, bijux-dna-api, bijux-dna-domain-compiler

## What
Points to the canonical boundary diagram and dependency rules.

## Why
Avoids boundary duplication across documents.

## Non-goals
- Restating dependency rules.

## Contracts
Enforced by:
- [docs/10-architecture/BOUNDARY_DIAGRAM.md](BOUNDARY_DIAGRAM.md)
- [docs/10-architecture/DEPENDENCY_RULES.md](DEPENDENCY_RULES.md)
- [crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs)
- [crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs)

## Executable Boundary Map
```boundaries
bijux-dna: bijux-dna-analyze bijux-dna-api bijux-dna-core bijux-dna-db-ena bijux-dna-domain-compiler bijux-dna-domain-vcf bijux-dna-environment bijux-dna-environment-qa bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-stages-vcf
bijux-dna-analyze: bijux-dna-bench bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-infra bijux-dna-pipelines bijux-dna-planner-bam bijux-dna-planner-fastq bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-api: bijux-dna-analyze bijux-dna-bench bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-engine bijux-dna-environment bijux-dna-environment-qa bijux-dna-infra bijux-dna-pipelines bijux-dna-planner-bam bijux-dna-planner-fastq bijux-dna-policies bijux-dna-runner bijux-dna-runtime bijux-dna-stage-contract bijux-dna-stages-vcf bijux-dna-testkit
bijux-dna-bench: bijux-dna-analyze bijux-dna-bench-model bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-bench-model: bijux-dna-analyze bijux-dna-core bijux-dna-domain-fastq bijux-dna-policies bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-core: bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-db-ena: bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-db-ref: bijux-dna-domain-vcf bijux-dna-policies
bijux-dna-dev: bijux-dna-api bijux-dna-core bijux-dna-db-ena bijux-dna-db-ref bijux-dna-infra bijux-dna-policies
bijux-dna-domain-bam: bijux-dna-core bijux-dna-policies bijux-dna-testkit
bijux-dna-domain-compiler: bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-infra
bijux-dna-domain-fastq: bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-domain-vcf: bijux-dna-policies
bijux-dna-engine: bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-environment: bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-environment-qa: bijux-dna-analyze bijux-dna-core bijux-dna-domain-fastq bijux-dna-environment bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-infra: bijux-dna-policies bijux-dna-testkit
bijux-dna-pipelines: bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-planner-bam: bijux-dna-core bijux-dna-domain-bam bijux-dna-infra bijux-dna-pipelines bijux-dna-policies bijux-dna-stage-contract bijux-dna-stages-bam bijux-dna-testkit
bijux-dna-planner-fastq: bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-infra bijux-dna-pipelines bijux-dna-policies bijux-dna-stage-contract bijux-dna-stages-fastq bijux-dna-testkit
bijux-dna-planner-vcf: bijux-dna-core bijux-dna-db-ref bijux-dna-domain-vcf bijux-dna-policies bijux-dna-stage-contract
bijux-dna-policies: bijux-dna-core bijux-dna-pipelines bijux-dna-runtime bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-runner: bijux-dna-core bijux-dna-environment bijux-dna-infra bijux-dna-policies bijux-dna-runtime
bijux-dna-runtime: bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-science: bijux-dna-infra bijux-dna-policies
bijux-dna-stage-contract: bijux-dna-core bijux-dna-policies bijux-dna-testkit
bijux-dna-stages-bam: bijux-dna-core bijux-dna-domain-bam bijux-dna-infra bijux-dna-policies bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-stages-fastq: bijux-dna-core bijux-dna-domain-fastq bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-stages-vcf: bijux-dna-core bijux-dna-db-ref bijux-dna-domain-vcf bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-testkit: bijux-dna-policies
```

## Examples
See [BOUNDARY_DIAGRAM.md](BOUNDARY_DIAGRAM.md) for the canonical diagram.

## Failure modes
Boundary violations fail CI dependency/effect policies.
