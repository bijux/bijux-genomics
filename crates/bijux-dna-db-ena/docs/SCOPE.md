# Scope

## What

Provide deterministic ENA selection and fetch primitives used by higher-level
corpus and benchmark workflows.

## In scope

- Query validation and ENA filereport URL building.
- ENA response parsing, normalization, and sample filtering.
- Download task planning under caller-provided output roots.
- Optional file transfer from ENA endpoints.
- Helper-binary manifest persistence at caller-provided manifest paths.

## Out of scope

- Pipeline execution.
- Scientific stage semantics.
- Runner, runtime, or environment orchestration.
- Top-level CLI command routing.
- Reference database management.

## Policy reference

- Workspace style and boundary policy:
  `README.md`, `README.md`, and
  repository `docs/40-policies/STYLE.md`.
