# SCIENTIFIC_DECISIONS

This document tracks default scientific thresholds and tool choices that affect interpretation and reproducibility.

## FASTQ
- `fastq.trim.min_len > 0` in all aDNA presets.
- `adapter_policy != none` in all aDNA presets.
- `fastq.merge` is required for paired reference aDNA unless explicitly disabled with justification.
- Reference-grade aDNA profile includes required stages for pre/post QC, trimming/filtering, merge, contamination hooks, and summary outputs.

## BAM
- aDNA BAM profiles require damage estimation stage, or explicit disable with justification.
- Sorting/indexing stages are required before downstream QC that depends on indexed BAM.

## VCF
- `vcf.stats` is required in VCF profiles.
- VCF stays experimental until schema + parser + smoke constraints are satisfied.

## Tooling and Pins
- Stable profiles may only use production tools with immutable pins.
- Floating tags and unresolved pins are forbidden in production readiness gates.

## Change Control
- Profile manifests are hash-addressed (`profile_hash`).
- Any defaults change must change profile hash and corresponding contract snapshot.
- Tool pin updates must update `configs/tool_registry.lock.sha256`.
