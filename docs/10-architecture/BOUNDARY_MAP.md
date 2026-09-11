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
bijux-dna: bijux-dna-bench-bam-evidence bijux-dna-bench-hpc bijux-dna-bench-fastq bijux-dna-bench-artifacts bijux-dna-bench-vcf-evidence bijux-dna-bench-genotypes bijux-dna-bench-variants bijux-dna-bench-haplotypes bijux-dna-bench-population bijux-dna-bench-workflows bijux-dna-bench-vcf bijux-dna-bench-corpus bijux-dna-analyze bijux-dna-api bijux-dna-core bijux-dna-db-ena bijux-dna-db-ref bijux-dna-domain-bam bijux-dna-domain-compiler bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-infra bijux-dna-planner-bam bijux-dna-planner-fastq bijux-dna-planner-vcf bijux-dna-policies bijux-dna-stage-contract bijux-dna-stages-fastq bijux-dna-stages-vcf bijux-dna-testkit
bijux-dna-analyze: bijux-dna-bench bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-infra bijux-dna-pipelines bijux-dna-planner-bam bijux-dna-planner-fastq bijux-dna-policies bijux-dna-runtime bijux-dna-testkit
bijux-dna-api: bijux-dna-analyze bijux-dna-bench bijux-dna-bench-model bijux-dna-core bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-engine bijux-dna-containers bijux-dna-containers-qa bijux-dna-infra bijux-dna-pipelines bijux-dna-planner-bam bijux-dna-planner-fastq bijux-dna-policies bijux-dna-runner bijux-dna-runtime bijux-dna-science bijux-dna-stage-contract bijux-dna-stages-bam bijux-dna-stages-fastq bijux-dna-stages-vcf bijux-dna-testkit
bijux-dna-bench: bijux-dna-bench-artifacts bijux-dna-analyze bijux-dna-bench-model bijux-dna-containers bijux-dna-core bijux-dna-db-ena bijux-dna-db-ref bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-infra bijux-dna-policies bijux-dna-runtime bijux-dna-science bijux-dna-stage-contract bijux-dna-testkit
bijux-dna-bench-corpus: bijux-dna-db-ref bijux-dna-domain-bam bijux-dna-domain-vcf bijux-dna-infra
bijux-dna-bench-artifacts: bijux-dna-bench-model bijux-dna-infra
bijux-dna-bench-hpc: bijux-dna-bench-model bijux-dna-core bijux-dna-infra
bijux-dna-bench-fastq: bijux-dna-domain-fastq bijux-dna-stage-contract bijux-dna-infra
bijux-dna-bench-genotypes: bijux-dna-bench-vcf-evidence bijux-dna-bench-artifacts bijux-dna-bench-corpus bijux-dna-bench-model bijux-dna-core bijux-dna-domain-vcf bijux-dna-infra bijux-dna-stages-vcf
bijux-dna-bench-variants: bijux-dna-bench-vcf-evidence bijux-dna-bench-artifacts bijux-dna-bench-model bijux-dna-bench-corpus bijux-dna-core bijux-dna-domain-vcf bijux-dna-infra bijux-dna-stages-vcf
bijux-dna-bench-haplotypes: bijux-dna-bench-vcf-evidence bijux-dna-bench-artifacts bijux-dna-bench-model bijux-dna-bench-corpus bijux-dna-core bijux-dna-infra bijux-dna-db-ref bijux-dna-domain-vcf bijux-dna-stage-contract bijux-dna-stages-vcf
bijux-dna-bench-population: bijux-dna-bench-vcf-evidence bijux-dna-bench-model bijux-dna-bench-corpus bijux-dna-core bijux-dna-infra bijux-dna-domain-vcf bijux-dna-stages-vcf
bijux-dna-bench-workflows: bijux-dna-bench-corpus bijux-dna-core bijux-dna-infra bijux-dna-domain-bam bijux-dna-domain-fastq bijux-dna-domain-vcf bijux-dna-stages-vcf bijux-dna-db-ref
bijux-dna-bench-vcf: bijux-dna-bench-artifacts bijux-dna-core bijux-dna-bench-vcf-evidence bijux-dna-bench-model bijux-dna-bench-corpus bijux-dna-db-ref bijux-dna-domain-vcf bijux-dna-stages-vcf bijux-dna-infra
bijux-dna-bench-bam-evidence: bijux-dna-core bijux-dna-infra
bijux-dna-bench-vcf-evidence:
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

Genotype benchmark composition calls `bijux-dna-bench-genotypes` with explicit
stage declarations. This owner depends on corpus alignment validation, VCF domain
and stage execution, benchmark model schemas, and shared artifact publication.
Calling, GL, diploid, pseudohaploid and likelihood-propagation workflows belong
there; the CLI acquires declarations and renders the resulting reports.

VCF benchmark evidence admission and command projection belong to
`bijux-dna-bench-vcf-evidence`. The independent contract has no workspace crate
edges. Full binding identities prevent cross-corpus parser joins; ambiguous and
contradictory observations fail admission. Acquisition and native execution remain
separate from the admitted evidence and its scientific interpretation.

VCF adapter evidence acquisition depends on the independent VCF evidence records
and command contracts. The adapter owner publishes compatible family reports and
projects their structured commands from one request-local observation context.
The evidence contract retains no dependency on the acquisition owner.

FASTQ synthetic report probes and typed report observations belong to `bijux-dna-bench-fastq`. Composition supplies governed plans; its normal dependency closure excludes application, planner and sibling benchmark services. The probe reports identify synthetic copy-through, empty-output and index-marker observations rather than backend execution.

Local watchdog simulation policy belongs to `bijux-dna-bench-hpc`. Typed DAG input admission, scheduling, scenario assessment and atomic report publication are independent of CLI and backend transport. Simulated evidence does not establish remote execution readiness.

Local submission readiness also belongs to `bijux-dna-bench-hpc`. Six explicit evidence interfaces isolate its99check orchestration from CLI, backend, corpus and campaign implementations. CLI composition maps canonical inputs and preserves feature refusal; persisted readiness schemas remain in benchmark models. The local campaign interface permits mock submission only.

BAM declaration evidence is owned by `bijux-dna-bench-bam-evidence`. Typed observation ports retain native domain/planner and shared cross-domain schema authority in their existing owners. The evidence crate owns coverage projection, report placement, exact binding consistency and local diagnostic publication, with no normal or test dependency on native BAM, planners, CLI, API or the benchmark umbrella. Declaration readiness alone does not establish executed payload validity or scientific substitutability.
