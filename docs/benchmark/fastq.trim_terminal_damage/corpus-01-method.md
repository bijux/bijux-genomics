# `fastq.trim_terminal_damage` corpus-01 method

## Intent

This benchmark measures the governed `fastq.trim_terminal_damage` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

The benchmark contract is:

- full corpus coverage: `20` normalized samples
- balanced cohort coverage: `5` ancient single-end, `5` ancient paired-end, `5` modern single-end, `5` modern paired-end
- full stage tool coverage: `adapterremoval`, `cutadapt`, and `seqkit`
- explicit damage policy contract:
  - `damage_mode=ancient`
  - `execution_policy=explicit_terminal_trim`
  - `trim_5p_bases=2`
  - `trim_3p_bases=2`

## Execution

Run the corpus benchmark from the Lunarc frontend against the workspace-configured corpus root:

```bash
python3 makes/bin/run_fastq_trim_terminal_damage_corpus_01.py \
  --repo-root . \
  --platform lunarc-apptainer \
  --damage-mode ancient \
  --execution-policy explicit_terminal_trim \
  --trim-5p-bases 2 \
  --trim-3p-bases 2
```

Render the published report set after the run completes:

```bash
python3 makes/bin/render_fastq_trim_terminal_damage_corpus_01_report.py \
  --repo-root .

python3 makes/bin/render_fastq_trim_terminal_damage_corpus_01_briefing.py \
  --docs-root docs/benchmark/fastq.trim_terminal_damage/corpus-01
```

The make aliases mirror the same flow:

```bash
make _benchmark-trim-terminal-damage-corpus-01 PLATFORM=lunarc-apptainer
make _benchmark-trim-terminal-damage-corpus-01-report
```

The runner and report renderer resolve the governed Lunarc corpus root and run root from [workspace.toml](/Users/bijan/bijux/bijux-dna/configs/bench/workspace.toml). Override `--corpus-root` only when you intentionally audit a non-governed mirror.

## Artifact Contract

The runner writes the execution manifest under the Lunarc run root:

- `run_manifest.json`
- `bench/trim_terminal_damage/<sample_id>/report.json`
- `bench/trim_terminal_damage/<sample_id>/bench.jsonl`
- `bench/trim_terminal_damage/<sample_id>/bench.sqlite`

The report renderers publish the doc set under [corpus-01](/Users/bijan/bijux/bijux-dna/docs/benchmark/fastq.trim_terminal_damage/corpus-01):

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
- mixed damage policy rows inside one corpus report
- missing per-sample tool rows
- backend report format drift between governed tools and native reports
- attempts to publish a report from a `--dry-run` manifest

## Interpretation

Use the resulting dossier for:

- comparing runtime cost between the governed terminal-damage backends
- comparing retained base fractions after terminal trimming
- comparing which tools reduce terminal C>T / G>A asymmetry more strongly
- identifying whether modern DNA behaves as a negative-control cohort under an ancient-DNA trim policy

Do not use the benchmark dossier alone to claim:

- final biological damage truth for the cohort
- final preprocessing policy outside the governed Lunarc platform
- that modern cohorts should receive damage-aware trimming in production
