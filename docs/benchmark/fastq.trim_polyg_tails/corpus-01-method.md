# `fastq.trim_polyg_tails` on `corpus-01`

## Intent

This benchmark measures the governed `fastq.trim_polyg_tails` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

The benchmark contract is:

- full corpus coverage: `20` normalized samples
- balanced cohort coverage: `5` ancient single-end, `5` ancient paired-end, `5` modern single-end, `5` modern paired-end
- full stage tool coverage: `fastp` and `bbduk`
- explicit chemistry contract: `polyx_preset=illumina_twocolor`
- explicit trim threshold: `min_polyg_run=10`

## Execution

Run the corpus benchmark from the Lunarc frontend against the materialized corpus root:

```bash
python3 makes/bin/run_fastq_trim_polyg_tails_corpus_01.py \
  --repo-root . \
  --corpus-root /home/bijan/bijux/corpus_01 \
  --platform lunarc-apptainer \
  --tools fastp,bbduk \
  --polyx-preset illumina_twocolor \
  --min-polyg-run 10
```

Render the published report set after the run completes:

```bash
python3 makes/bin/render_fastq_trim_polyg_tails_corpus_01_report.py \
  --repo-root . \
  --corpus-root /home/bijan/bijux/corpus_01

python3 makes/bin/render_fastq_trim_polyg_tails_corpus_01_briefing.py \
  --docs-root docs/benchmark/fastq.trim_polyg_tails/corpus-01
```

The make aliases mirror the same flow:

```bash
make _benchmark-trim-polyg-corpus-01 PLATFORM=lunarc-apptainer CORPUS_ROOT=/home/bijan/bijux/corpus_01
make _benchmark-trim-polyg-corpus-01-report CORPUS_ROOT=/home/bijan/bijux/corpus_01
```

## Artifact Contract

The runner writes the execution manifest under the Lunarc run root:

- `run_manifest.json`
- `bench/trim_polyg_tails/<sample_id>/report.json`
- `bench/trim_polyg_tails/<sample_id>/bench.jsonl`
- `bench/trim_polyg_tails/<sample_id>/bench.sqlite`

The report renderers publish the doc set under [corpus-01](/Users/bijan/bijux/bijux-dna/docs/benchmark/fastq.trim_polyg_tails/corpus-01):

- `summary.json`
- `sample_results.csv`
- `tool_runtime_summary.csv`
- `cohort_runtime_summary.csv`
- `sample_runtime_outliers.csv`
- `lunarc.md`

## Guardrails

The corpus scripts intentionally fail when the run is incomplete or incoherent.

They reject:

- corpus drift from the committed cohort balance
- partial tool rosters
- mixed `polyx_preset` values inside one corpus report
- mixed `min_polyg_run` values inside one corpus report
- missing per-sample tool rows
- backend report format drift between governed tools and native reports

## Interpretation

Use the resulting dossier for:

- comparing runtime cost between `fastp` and `bbduk`
- comparing retained base fractions after governed polyG trimming
- identifying which cohort segments actually exercise polyG cleanup work
- identifying the most expensive or most aggressively trimmed samples

Do not use the benchmark dossier alone to claim:

- biological truth about damage or sequencing chemistry
- generalized performance outside the governed Lunarc platform
- final trimming policy for datasets with a different sequencer chemistry contract
