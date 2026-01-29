# Separation of Concerns Contract

This document defines ownership boundaries for each crate. The intent is to prevent execution logic from leaking into planning, and to keep CLI as a thin adapter.

## bijux-core

Owns:
- Core data types and shared serialization formats.
- Cross-cutting utilities and invariants.

Does not own:
- Domain models or stage planning.
- Execution or tool invocation.

Allowed deps:
- None (foundation layer).

## bijux-domain-*

Owns:
- Domain models, contracts, and validation logic for the domain.

Does not own:
- Tool execution or runtime concerns.
- CLI UX.

Allowed deps:
- bijux-core

## bijux-stages-fastq

Owns:
- Stage planning, defaults, and output contracts for FASTQ.
- Stage registry for FASTQ.

Does not own:
- Execution or container orchestration.
- CLI UX or reporting.

Allowed deps:
- bijux-domain-fastq
- bijux-core

## bijux-engine

Owns:
- Execution, observation, and artifact materialization.
- Runtime metrics and tool invocation.

Does not own:
- Stage planning or defaults.
- CLI UX.

Allowed deps:
- bijux-core
- bijux-environment

## bijux-cli

Owns:
- UX, argument parsing, and error mapping.
- Routing CLI commands to stages planning + engine execution.

Does not own:
- Planning logic, defaults, or execution.

Allowed deps:
- bijux-engine
- bijux-stages-fastq
- bijux-core

## bijux-analyze / bijux-bench

Owns:
- Reporting, benchmarking, and analysis over produced artifacts.

Does not own:
- Execution or planning logic.

Allowed deps:
- bijux-core
- bijux-stages-fastq
