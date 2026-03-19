# PARAMS

Canonicalization normalizes:
- ordering of keys
- float formatting
- path normalization

This ensures stable hashing and comparisons.

## Checklist: add a new stage param
- Update param schema and canonicalization in `src/params/*`.
- Update invariant expectations in `src/invariants/*`.
- Update metric semantics in `src/metrics/*`.
- Refresh stage contract snapshots in `tests/contracts/stage_contract_snapshots.rs`.

## Tool-specific trim params
The FASTQ trim domain now exposes typed tool models in `src/params/trim.rs`:
- `SkewerTrimParamsV1`
- `LeeHomTrimParamsV1`
- `AlienTrimmerParamsV1`
- `FastxClipperParamsV1`

Shared axes are modeled explicitly:
- adapter modes
- minimum length (bp)
- quality trim mode and cutoff (Phred)
- overlap/collapse behavior
- paired/single read handling

`LeeHomTrimParamsV1` also captures overlap-specific controls for ancient-DNA style merge/collapse behavior.

## Stage Param Types
FASTQ stage definitions use stage-specific parameter structs instead of a single validate model:

| Stage ID | Param Type | Meaning |
| --- | --- | --- |
| `fastq.validate_reads` | `ValidateEffectiveParams` | input FASTQ structural validation controls |
| `fastq.profile_reads` | `FastqStatsParams` | neutral read statistics collection controls |
| `fastq.correct_errors` | `FastqCorrectParams` | error-correction controls |
| `fastq.extract_umis` | `FastqUmiParams` | UMI extraction/normalization controls |
| `fastq.detect_adapters` | `DetectAdaptersEffectiveParams` | report-only adapter evidence controls |
| `fastq.trim_reads` | `TrimEffectiveParams` | adapter/quality/length trimming controls |
| `fastq.filter_reads` | `FilterEffectiveParams` | contamination and complexity filtering controls |
| `fastq.merge_pairs` | `MergeEffectiveParams` | paired-end overlap merge controls |
| `fastq.index_reference` | `ReferenceIndexEffectiveParams` | reference-index build controls |
| `fastq.deplete_host` | `HostDepletionEffectiveParams` | host-reference depletion controls |
| `fastq.deplete_reference_contaminants` | `ReferenceContaminantEffectiveParams` | reference-contaminant depletion controls |
| `fastq.deplete_rrna` | `RrnaEffectiveParams` | rRNA screen controls |
| `fastq.screen_taxonomy` | `ScreenEffectiveParams` | contaminant taxonomy screen controls |
| `fastq.report_qc` | `QcPostEffectiveParams` | post-processing QC report controls |
