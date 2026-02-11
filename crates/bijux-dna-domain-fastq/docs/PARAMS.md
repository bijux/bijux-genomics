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
| `fastq.validate_pre` | `ValidateEffectiveParams` | input FASTQ structural validation controls |
| `fastq.stats_neutral` | `FastqStatsParams` | neutral read statistics collection controls |
| `fastq.correct` | `FastqCorrectParams` | error-correction controls |
| `fastq.umi` | `FastqUmiParams` | UMI extraction/normalization controls |
| `fastq.detect_adapters` | `DetectAdaptersEffectiveParams` | adapter discovery controls |
| `fastq.trim` | `TrimEffectiveParams` | adapter/quality/length trimming controls |
| `fastq.filter` | `FilterEffectiveParams` | contamination and complexity filtering controls |
| `fastq.merge` | `MergeEffectiveParams` | paired-end overlap merge controls |
| `fastq.rrna` | `RrnaEffectiveParams` | rRNA screen controls |
| `fastq.screen` | `ScreenEffectiveParams` | contaminant taxonomy screen controls |
| `fastq.qc_post` | `QcPostEffectiveParams` | post-processing QC report controls |
| `fastq.preprocess` | `PreprocessEffectiveParams` | stage orchestration controls |
