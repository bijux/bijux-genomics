# What Is Bijux DNA

## What
Bijux DNA is a contract‑first genomics pipeline system for FASTQ and BAM processing. It produces deterministic execution graphs, reproducible run artifacts, and structured reports.

## Why
Scientific pipelines need auditability, stable inputs/outputs, and explainable tool selection. Bijux DNA enforces this with strict contracts, canonical serialization, and policy‑backed boundaries.

## Non-goals
- Replacing domain‑specific scientific judgment.
- Supporting every tool in the ecosystem.
- Running arbitrary scripts as pipeline stages.

## Contracts
- ExecutionGraph
- StageSpec
- RunManifest
- ToolInvocation
- MetricsEnvelope

## Examples
- A FASTQ preprocessing run produces a graph, per‑stage metrics envelopes, and a report bundle.
- A BAM pipeline run records stage outputs and tool invocations for reproducibility.

## Failure modes
- Missing declared artifacts fail fast with ContractError.
- Invalid pipeline IDs or profiles fail planning with PlanError.
