# Boundaries

Each crate below lists responsibilities, forbidden responsibilities, and allowed dependencies.

## bijux-core
- Responsibilities: core identifiers, run metadata/events, metrics schemas, shared utilities.
- Forbidden: domain-specific logic, tool execution, CLI parsing.
- Allowed deps: serde/serde_json, chrono, uuid, tracing, std-only utilities.

## bijux-cli
- Responsibilities: CLI UX, argument parsing, wiring stages to execution, emitting artifacts.
- Forbidden: domain contracts/logic beyond staging, engine internals, tool-specific parsing.
- Allowed deps: bijux-core, bijux-stages-fastq, bijux-engine (API only), bijux-analyze, bijux-environment.

## bijux-stages-fastq
- Responsibilities: stage planning, contracts front door, default config resolution.
- Forbidden: execution/runtime, container/runner logic, IO side-effects.
- Allowed deps: bijux-core, bijux-domain-*, serde/serde_json.

## bijux-engine
- Responsibilities: execution orchestration, pipeline wiring, runner selection.
- Forbidden: domain semantics, CLI parsing, report formatting.
- Allowed deps: bijux-core, bijux-environment, serde/serde_json.

## bijux-environment
- Responsibilities: platform specs, container/runner discovery, environment inspection.
- Forbidden: domain logic, CLI UX.
- Allowed deps: bijux-core, serde/serde_yaml, tracing.

## bijux-analyze
- Responsibilities: metrics validation, comparisons, report generation, selection logic.
- Forbidden: execution/runtime, CLI parsing.
- Allowed deps: bijux-core, serde/serde_json, rusqlite.

## bijux-benchmark
- Responsibilities: benchmark harnesses and gates.
- Forbidden: engine execution, domain parsing.
- Allowed deps: bijux-core, bijux-analyze.

## bijux-domain-fastq
- Responsibilities: FASTQ contracts, invariants, pipeline specs, data models.
- Forbidden: engine execution, CLI UX, environment discovery.
- Allowed deps: bijux-core, serde/serde_json, serde_yaml.

## bijux-domain-bam
- Responsibilities: BAM contracts and domain models.
- Forbidden: execution/runtime, CLI UX.
- Allowed deps: bijux-core, serde/serde_json.

## bijux-domain-vcf
- Responsibilities: VCF contracts and domain models.
- Forbidden: execution/runtime, CLI UX.
- Allowed deps: bijux-core, serde/serde_json.

## bijux-domain-dummy
- Responsibilities: sample domain examples/testing fixtures.
- Forbidden: engine/CLI logic.
- Allowed deps: bijux-core.
