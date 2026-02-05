# bijux-dna v1 Scope Lock

## What bijux-dna v1 does
- Defines FASTQ stages with fixed contracts (validate, trim, merge, correct, filter, qc_post, umi, screen).
- Provides containerized execution with reproducible inputs and outputs.
- Records metrics and execution context for benchmarking.
- Supports deterministic QA and benchmarking runs on real FASTQ datasets.

## What bijux-dna v1 does not do
- No new tools added beyond the frozen set for v1.
- No automatic pipeline DAG selection or auto-run orchestration.
- No accuracy scoring beyond deterministic derived metrics.
- No parameter sweeps or production presets.
- No cross-domain (non-FASTQ) pipelines.

## Frozen artifacts
- Contracts in `docs/contracts/fastq/`.
- Metric definitions in `docs/metrics/`.
- Image QA protocol and datasets under `lab/corpus/fastq`.

## Change control
- Any new tool, metric, or stage requires a v2 plan.
- v1 scope changes are blocked unless explicitly approved in a v2 roadmap.
