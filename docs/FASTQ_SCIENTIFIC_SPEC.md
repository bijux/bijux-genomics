FASTQ Scientific Spec (v1)
==========================

Purpose
-------
Define the scientific method behind the FASTQ pipeline: assumptions, defaults, gates, and how to interpret results. This is the contract that makes the pipeline a method, not just software.

Scope
-----
Applies to FASTQ stages:
validate_pre → trim → filter → merge (optional) → qc_post (optional) → stats_neutral.

Core assumptions
----------------
- Input FASTQ files are syntactically valid and represent the intended sample.
- Reference assets (adapters/contaminants) are curated and versioned.
- All outputs are deterministic given inputs + tool versions + parameters.

Recommended defaults
--------------------
- Tool tier policy: gold-only unless `--allow-silver` or `--allow-experimental`.
- Adapter presets: illumina-default unless a scientific preset changes it.
- QC post-checks: enabled by default (use `--no-qc-post` to disable).
- Merge policy: only merge when suitable (or forced).
- Contaminant removal: off by default; can be enabled per preset or `--enable-contaminant-removal`.

Scientific presets
------------------
- ancient_dna: favor adapter sensitivity (ssdna), contamination removal on, no forced merge.
- amplicon: favor merging and aggressive adapter trimming.
- metagenomic: contamination removal on, no forced merge.
- wgs_standard: balanced defaults, no forced overrides.

Gates & assertions
------------------
Each stage emits assertions with pass/warn/fail:
- Reads/bases monotonicity (no increases after filtering).
- Plausible ranges for mean Q and GC% (bounded).
- Delta bounds (mean Q and GC% deltas).
- Retention metrics must be in [0, 1].

Quality gate policy
-------------------
Stages validate_pre/trim/filter/qc_post apply a gate:
- read_retention warn < 0.7, fail < 0.4
- mean_q warn < 20, fail < 15
- mean_q_delta warn < -1
Gate decisions are emitted to telemetry and facts.

Interpreting the report
-----------------------
- Decision trace: why each tool/step was chosen; missing metrics are explicit.
- Stage confidence: score derived from assertion results + missing metrics + tool exit status.
- QC delta: pre/post changes in key metrics.
- Contamination summary: top references and percent removed.
- Tool tier: reported for each stage (gold/silver/experimental).

Limitations
-----------
- Uncertainty intervals are reported only when benchmark replicates are present.
- Contaminant removal currently single-path and optional.
- Presets tune defaults but do not replace human oversight.

Report & facts versioning
-------------------------
- Facts schema: `bijux.facts.v1`
- Report schema: `bijux.report.v1`
- Compatibility rule: unknown fields are ignored safely.
- Migration: new fields must be additive; breaking changes require a new version with a clear upgrade path.

When to override presets
------------------------
- Known library kits with non-standard adapters.
- Datasets with expected low complexity or extreme GC.
- Special pipelines (amplicon, ancient DNA) requiring bespoke thresholds.
