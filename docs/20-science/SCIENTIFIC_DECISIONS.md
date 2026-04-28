# Scientific Decisions

## Purpose
This document records the cross-domain scientific decisions that remain normative above individual tool manifests and per-domain science ledgers.

## Scope
- Cross-domain scientific defaults and gating rules that affect FASTQ, BAM, and VCF workflows.
- Decision rules that stay normative above per-tool manifests and evidence ledgers.

## Non-goals
- Replacing per-domain scientific specs or detailed method ledgers.
- Claiming citation closure or promotion readiness for every governed tool.

## Contracts
- Domain-specific scientific specs remain the detailed authority for each surface.
- Cross-domain defaults recorded here stay normative until a governed update changes them.

Publication-scoped scientific artifacts are referenced via [the current publication index](../../assets/publications/adna-methods-2024/index.md); governed publication-asset handling lives in [Publication Assets](PUBLICATION_ASSETS.md).

## FASTQ
- Authority: [FASTQ Scientific Spec](fastq/SCIENTIFIC_SPEC.md).
- `fastq.trim_reads.min_len > 0` in all aDNA presets.
- `adapter_policy != none` in all aDNA presets.
- `fastq.merge_pairs` is required for paired reference aDNA unless explicitly disabled with justification.
- Reference-grade aDNA profile includes required stages for pre/post QC, trimming/filtering, merge, contamination hooks, and summary outputs.

## BAM
- Authority: [BAM Methodological Intent](bam/METHODOLOGICAL_INTENT.md).
- aDNA BAM profiles require damage estimation stage, or explicit disable with justification.
- Sorting/indexing stages are required before downstream QC that depends on indexed BAM.

## VCF
- Authority: [VCF Science Index](vcf/index.md).
- `vcf.stats` is required in VCF profiles.
- VCF stays experimental until schema + parser + smoke constraints are satisfied.

## Tooling and Pins
- Stable profiles may only use production tools with immutable pins.
- Floating tags and unresolved pins are forbidden in production readiness gates.

## Change Control
- Profile manifests are hash-addressed (`profile_hash`).
- Any defaults change must change profile hash and corresponding contract snapshot.
- Tool pin updates must update `configs/ci/registry/tool_registry_lock.sha256`.
