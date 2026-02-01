# FASTQ Gold Pipeline Spec (v1)

## Scope
This document describes the default FASTQ pipeline behavior, intended knobs, and outputs.
It is the reference for product, QA, and downstream analysis expectations.

## Default stages
Required stages (default order):
1) validate-pre
2) trim
3) filter
4) stats-neutral
5) qc-post

Optional stages (enabled by policy or flags):
- merge (paired-end only, requires suitability or --force-merge)
- correct
- umi
- screen (enabled when contaminant preset is set)

## Tool tiers (selection policy)
Tool tiers are derived from manifests:
- gold = authoritative
- silver = diagnostic
- experimental = experimental

Default selection: gold only.
Override: `--allow-experimental` (includes silver + experimental).

## Merge suitability
Merge is enabled only when:
- paired-end reads are present, and
- suitability check passes (or `--force-merge`).

The merge decision trace is emitted into telemetry and run summary for auditability.

## Adapter policy
Supported presets:
- illumina-default (default)
- ssdna
- none
- custom-file (from `--adapter-bank-file`)

Adapter auto-detect is sourced from FastQC/MultiQC outputs when available; currently
it is warn-only and never overrides explicit user choices.

## Contaminant screening
Contaminant screening is enabled when a contaminant preset is provided.
The screen stage emits contamination rate and summary entries; reports surface:
- reads removed
- percent removed
- top taxa/reference summary

## Reporting outputs
Artifacts are written under the run artifacts tree:
- facts.jsonl
- report.json
- run_summary.json
- telemetry/events.jsonl

Report sections include:
- pipeline overview
- key findings
- QC delta (validate_pre vs qc_post)
- reproducibility block
- data contract validation summary

## Failure taxonomy
Failure kinds map to classes:
- data error: DataInvalid, ContractViolation
- tool error: ToolExit, ObserverParse
- environment error: ResourceExhaustion, ImageError

This taxonomy is used in remediation guidance and report summaries.

## Out of scope
- Domain-specific read correction beyond configured stages
- Cross-run normalization and meta-analysis
- Clinical interpretations
