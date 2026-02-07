# Workspace Boundaries

Owner: Architecture
Scope: Executable dependency allowlist for workspace crates.
Last reviewed: 2026-02-07
Contract version: v1

## Contract
```boundaries
bijux: bijux-api bijux-core bijux-infra bijux-policies
bijux-analyze: bijux-benchmark bijux-core bijux-domain-bam bijux-domain-fastq bijux-infra bijux-pipelines bijux-planner-bam bijux-planner-fastq bijux-policies bijux-runtime
bijux-api: bijux-analyze bijux-benchmark bijux-core bijux-domain-bam bijux-domain-fastq bijux-engine bijux-environment bijux-environment-qa bijux-infra bijux-pipelines bijux-planner-bam bijux-planner-fastq bijux-policies bijux-runner bijux-runtime bijux-stage-contract
bijux-benchmark: bijux-analyze bijux-benchmark-model bijux-core bijux-domain-bam bijux-domain-fastq bijux-infra bijux-policies bijux-runtime
bijux-benchmark-model: bijux-analyze bijux-core bijux-policies
bijux-core: bijux-infra bijux-policies
bijux-domain-bam: bijux-core bijux-policies
bijux-domain-fastq: bijux-core bijux-infra bijux-policies
bijux-engine: bijux-core bijux-infra bijux-policies bijux-runtime
bijux-environment: bijux-core bijux-infra bijux-policies bijux-runtime
bijux-environment-qa: bijux-analyze bijux-core bijux-domain-fastq bijux-environment bijux-infra bijux-policies bijux-runtime
bijux-infra: bijux-policies
bijux-pipelines: bijux-core bijux-domain-bam bijux-domain-fastq bijux-policies
bijux-planner-bam: bijux-core bijux-domain-bam bijux-infra bijux-pipelines bijux-policies bijux-stage-contract bijux-stages-bam
bijux-planner-fastq: bijux-core bijux-domain-bam bijux-domain-fastq bijux-infra bijux-pipelines bijux-policies bijux-stage-contract bijux-stages-fastq
bijux-policies:
bijux-runner: bijux-core bijux-environment bijux-infra bijux-policies bijux-runtime
bijux-runtime: bijux-core bijux-infra bijux-policies
bijux-stage-contract: bijux-core bijux-policies
bijux-stages-bam: bijux-core bijux-domain-bam bijux-infra bijux-policies bijux-stage-contract
bijux-stages-fastq: bijux-core bijux-domain-fastq bijux-infra bijux-policies bijux-runtime bijux-stage-contract bijux-testkit
bijux-testkit: bijux-policies
```
