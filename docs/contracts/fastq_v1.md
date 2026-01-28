# FASTQ v1 Contract

This document freezes the FASTQ v1 contract for Bijux.

## Stages

- validate
- trim
- filter
- merge
- correct
- stats
- qc_post
- umi
- screen
- preprocess (meta)

## Artifacts

- FastqSE
- FastqPE
- FastqStats

## Metrics

Metrics are defined by stage and must comply with the stage metric spec and invariants. See:

- docs/domains/fastq_contract.md
- docs/domains/fastq.md

## Invariants

- Stage boundary invariants are defined in `bijux-domain-fastq/src/domain.rs`.
- Stage metric invariants are defined in `bijux-domain-fastq/src/metrics/spec.rs`.

## Compatibility Guarantees

- Canonical stage order is stable for v1.
- Tool manifests must declare roles and are filtered by role in strict mode.
- Output compatibility is enforced by stage contracts.
