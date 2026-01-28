# Architecture North Star

This document is the architectural boundary for Bijux. It defines the separation between engine, domains, and bench, and explains how data flows and where invariants live.

## Components

Engine
- Executes plans and tools.
- Tracks resources and emits raw metrics.
- Enforces generic invariants (non-domain).
- Never knows domain semantics (no FASTQ, BAM, VCF).

Domain
- Owns domain semantics: stage contracts, invariants, compatibility rules.
- Defines artifact kinds, metrics schema, and deltas.
- Provides adapters for validation and analysis.

Bench
- Interprets metrics.
- Ranks, gates, and compares results.
- Never runs tools or mutates execution.

CLI
- Parses intent, validates flags.
- Calls engine/domain/bench APIs.
- Does not contain pipeline logic.

## Data Flow

1) CLI parses arguments and constructs a request.
2) Domain validates inputs and compatibility.
3) Engine composes an ExecutionPlan and executes it.
4) Observer emits metrics and explain artifacts.
5) Domain validates outputs and computes deltas.
6) Bench analyzes metrics for ranking/gates/comparisons.

## Invariants

- Domain invariants are enforced at stage boundaries.
- Execution is reproducible via manifests and explain artifacts.
- Metrics are schema-validated and versioned.

## Separation Rules

- Engine must not import domain crates.
- Domains must not embed execution logic.
- Bench must not call into execution or policy selection.

## Why This Separation Exists

- It prevents semantic drift across domains.
- It makes benchmarking and analysis authoritative.
- It ensures future BAM/VCF domains can be added without engine changes.
