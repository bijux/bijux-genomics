# FASTQ Lifecycle Tiers

This document defines the FASTQ lifecycle tiers for Bijux. It is the canonical reference for what is core, augmenting, and meta for FASTQ.

## Tier 1 — Core (mandatory)

These stages are required for a compliant FASTQ pipeline:

- validate
- trim
- merge
- correct
- filter
- stats

## Tier 2 — Augmenting

These stages add additional signals or post-processing and are optional:

- qc2
- umi
- screen

## Tier 3 — Meta

Meta stages orchestrate other stages and do not introduce new semantics:

- preprocess
