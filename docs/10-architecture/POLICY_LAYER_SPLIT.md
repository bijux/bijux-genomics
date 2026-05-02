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

## Purpose
This document describes the governed intent and operator-facing meaning of this surface.

## Scope
The scope is limited to repository-owned behavior, contracts, and evidence paths for this topic.

## Non-goals
This document does not redefine source-of-truth schemas, code ownership boundaries, or release policy outside this surface.

## Contracts
Claims here are valid only when they remain consistent with governed configs, domain authorities, and policy checks.

