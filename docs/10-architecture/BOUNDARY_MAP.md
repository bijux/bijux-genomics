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
bijux-dna: bijux-dna-bench-haplotypes bijux-dna-bench-population bijux-dna-bench-workflows bijux-dna-bench-vcf bijux-dna-bench-corpus bijux-dna-analyze bijux-dna-api bijux-dna-core bijux-dna-db-ena bijux-dna-db-ref bijux-dna-domain-bam bijux-dna-domain-compiler bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-infra bijux-dna-planner-bam bijux-dna-planner-fastq bijux-dna-planner-vcf bijux-dna-policies bijux-dna-stage-contract bijux-dna-stages-fastq bijux-dna-stages-vcf bijux-dna-testkit
bijux-dna-analyze: bijux-dna-bench bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-infra bijux-dna-pipelines bijux-dna-planner-bam bijux-dna-planner-fastq bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-api: bijux-dna-analyze bijux-dna-bench bijux-dna-bench-model bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-engine bijux-dna-containers bijux-dna-containers-qa bijux-dna-infra bijux-dna-pipelines bijux-dna-planner-bam bijux-dna-planner-fastq bijux-dna-policies bijux-dna-runner bijux-dna-runtime bijux-dna-science bijux-dna-stage-contract bijux-dna-stages-bam bijux-dna-stages-fastq bijux-dna-stages-vcf bijux-dna-testkit
bijux-dna-bench: bijux-dna-analyze bijux-dna-bench-model bijux-dna-containers bijux-dna-core bijux-dna-db-ena bijux-dna-db-ref bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-science bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-bench-corpus: bijux-dna-db-ref bijux-dna-domain-bam bijux-dna-domain-vcf bijux-dna-infra
bijux-dna-bench-haplotypes: bijux-dna-bench-model bijux-dna-bench-corpus bijux-dna-core bijux-dna-infra bijux-dna-db-ref bijux-dna-domain-vcf bijux-dna-stage-contract bijux-dna-stages-vcf
bijux-dna-bench-population: bijux-dna-bench-model bijux-dna-bench-corpus bijux-dna-core bijux-dna-infra bijux-dna-domain-vcf bijux-dna-stages-vcf
bijux-dna-bench-workflows: bijux-dna-bench-corpus bijux-dna-core bijux-dna-infra bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-stages-vcf bijux-dna-db-ref
bijux-dna-bench-vcf: bijux-dna-bench-model bijux-dna-bench-corpus bijux-dna-db-ref bijux-dna-domain-vcf bijux-dna-stages-vcf bijux-dna-infra
bijux-dna-bench-model: bijux-dna-analyze bijux-dna-core bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-policies bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-core: bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-db-ena: bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-db-ref: bijux-dna-domain-vcf bijux-dna-policies
bijux-dna-dev: bijux-dna-api bijux-dna-bench bijux-dna-containers bijux-dna-core bijux-dna-db-ena bijux-dna-db-ref bijux-dna-domain-compiler bijux-dna-infra bijux-dna-policies bijux-dna-runner bijux-dna-runtime bijux-dna-stage-contract
bijux-dna-domain-bam: bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-domain-compiler: bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-infra
bijux-dna-domain-fastq: bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-domain-vcf: bijux-dna-core bijux-dna-policies
bijux-dna-engine: bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-containers: bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-process bijux-dna-runtime bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-containers-qa: bijux-dna-analyze bijux-dna-bench-model bijux-dna-core bijux-dna-domain-fastq bijux-dna-containers bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-infra: bijux-dna-policies bijux-dna-testkit
bijux-dna-pipelines: bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-planner-bam: bijux-dna-core bijux-dna-domain-bam bijux-dna-infra bijux-dna-pipelines bijux-dna-policies bijux-dna-stage-contract bijux-dna-stages-bam bijux-dna-testkit
bijux-dna-planner-fastq: bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-infra bijux-dna-pipelines bijux-dna-policies bijux-dna-stage-contract bijux-dna-stages-fastq bijux-dna-testkit
bijux-dna-planner-vcf: bijux-dna-core bijux-dna-db-ref bijux-dna-domain-vcf bijux-dna-policies bijux-dna-stage-contract
bijux-dna-policies: bijux-dna-containers bijux-dna-core bijux-dna-pipelines bijux-dna-runtime bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-process:
bijux-dna-runner: bijux-dna-bench-model bijux-dna-containers bijux-dna-containers-qa bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-process bijux-dna-runtime
bijux-dna-runtime: bijux-dna-bench-model bijux-dna-core bijux-dna-infra bijux-dna-policies bijux-dna-testkit
bijux-dna-science: bijux-dna-bench-model bijux-dna-core bijux-dna-domain-vcf bijux-dna-infra bijux-dna-policies
bijux-dna-stage-contract: bijux-dna-core bijux-dna-policies bijux-dna-testkit
bijux-dna-stages-bam: bijux-dna-core bijux-dna-domain-bam bijux-dna-infra bijux-dna-policies bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-stages-fastq: bijux-dna-core bijux-dna-domain-fastq bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-stages-vcf: bijux-dna-core bijux-dna-db-ref bijux-dna-domain-vcf bijux-dna-infra bijux-dna-policies bijux-dna-runner bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-testkit: bijux-dna-policies
```

## Examples
See [BOUNDARY_DIAGRAM.md](BOUNDARY_DIAGRAM.md) for the canonical diagram.

## Failure modes
Boundary violations fail CI dependency/effect policies.

`bijux-dna-process` owns OS process capture and cancellation with no workspace production dependencies. Containers and runner share that transport while retaining separate command meaning, evidence interpretation and lifecycle policy. Independent compilation follows the leaf graph; elapsed build gains require measurement.
