# Policy Layer Split

Owner: Architecture  
Scope: scientific policy versus operational policy separation  
Last reviewed: 2026-04-30  
Contract version: v1

## Scientific policy

Scientific policy owns:

- reference compatibility rules
- sample assumptions and cohort assumptions
- advisory versus enforced scientific findings
- scientific thresholds that change interpretation

Authoritative sources live in `domain/`, `science/specs/`, and science-facing docs.

## Operational policy

Operational policy owns:

- executor choice and runtime platform
- retries, timeouts, and queue behavior
- storage/layout knobs
- run base directories and logging controls

Authoritative sources live under `configs/runtime/`, `configs/logging/`, and `configs/hpc/`.

## Guardrail

Operational profile files must not silently redefine scientific thresholds, reference
compatibility, sample assumptions, or advisory/enforced scientific findings. Those changes must
happen in scientific/domain authorities and be reviewed as scientific contract changes.

## Scope
This document defines the operational or architecture surface for this workflow surface.

## Non-goals
- Replacing crate-level implementation details or test contracts.

## Contracts
- Changes to this surface must stay aligned with governed checks and generated references.

## Purpose
This document records the durable intent and enforcement boundary for this surface.
