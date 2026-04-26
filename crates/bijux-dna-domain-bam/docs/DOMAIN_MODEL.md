# bijux-dna-domain-bam Domain Model

This crate owns BAM domain truth: stage ids, stage order, effective params, metric schemas, artifact policies, and invariant semantics.

## Stage phases

### Pre

Purpose: alignment and basic QC readiness.
Required artifacts include alignments and basic reports.
Required metrics include alignment rate, mapping quality distributions, and idxstats coverage summaries.

### Core

Purpose: canonical BAM outputs and core QC.
Required artifacts include sorted BAM, BAI, deduplicated BAM, and audit reports.
Required metrics include duplicate rate, coverage breadth, coverage depth, insert-size summaries, GC bias, and complexity estimates.

### Downstream

Purpose: interpretive assessments and verdicts.
Required artifacts include interpretive reports.
Required metrics include damage, contamination, authenticity, sex inference, genotyping, haplogroups, kinship sufficiency, recalibration, and bias mitigation.

## Params

- Alignment params cover presets, seed length, mismatch thresholds, reference inputs, and read-group behavior.
- Pre-QC and filtering params cover validation strictness, minimum length, and mapping-quality thresholds.
- Core params cover duplicate marking, complexity projection, coverage windows, damage thresholds, and report generation.
- Downstream params cover authenticity, contamination scope, sex inference, bias mitigation, recalibration, haplogroups, genotyping, and kinship thresholds.

Defaults are chosen for aDNA sensitivity and deterministic contract generation.

## Effects

The crate is allowed to perform pure deterministic computation over source-controlled data and explicit fixture files.
It must not spawn processes, perform network access, write generated configs, or inspect runtime environments.

## Change rules

- Changing a public contract field, stage id, metric id, param id, or serialized JSON shape is breaking.
- Breaking changes require explicit review plus snapshot and fixture updates in the same change set.
- Nonbreaking additions must update the stage, params, metrics, docs, fixtures, and tests that describe the new domain concept.
- Silent contract drift is a test failure, not a documentation-only concern.

## Authority

Source modules should reference this document for semantic boundaries and avoid duplicating broader domain prose in comments.
