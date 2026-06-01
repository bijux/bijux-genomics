# bijux-dna Commands

This file is the single source of truth for commands managed by the `bijux-dna` crate. Parser types
live under `src/commands/cli/parse/`, routing lives under `src/commands/router/`, and command
adapters live under `src/commands/*/`.

Commands listed here are owned by this crate even when their durable behavior is delegated to
`bijux-dna-api` or support crates.

## Stable Operator Commands

### Environment
- `bijux-dna env images`
- `bijux-dna env info`
- `bijux-dna env doctor`
- `bijux-dna env list`
- `bijux-dna env export-json`
- `bijux-dna env export-containers --json`
- `bijux-dna env export-hpc`
- `bijux-dna env sif-inventory`
- `bijux-dna env ensure`
- `bijux-dna env ensure-images`
- `bijux-dna env lint-apptainer-defs`
- `bijux-dna env smoke`
- `bijux-dna env prep`
- `bijux-dna env apptainer-qa-matrix`

### Registry
- `bijux-dna registry list-tools`
- `bijux-dna registry list-stages`
- `bijux-dna registry show-tool`
- `bijux-dna registry show-stage`
- `bijux-dna registry show`
- `bijux-dna registry export-json`
- `bijux-dna registry export-containers --json`
- `bijux-dna registry coverage-matrix`
- `bijux-dna registry validate-tool`
- `bijux-dna registry audit`
- `bijux-dna registry doctor`
- `bijux-dna registry promote`
- `bijux-dna registry lint`

### Corpus
- `bijux-dna corpus materialize`
- `bijux-dna corpus normalize`
- `bijux-dna corpus validate`
- `bijux-dna corpus list`
- `bijux-dna corpus diff`

### Status
- `bijux-dna status`

### FASTQ Run Surface
The public FASTQ command family is mounted under `run`.

- `bijux-dna run list-stages`
- `bijux-dna run stages`
- `bijux-dna run doctor`
- `bijux-dna run list-tools`
- `bijux-dna run explain`
- `bijux-dna run validate-pre`
- `bijux-dna run trim`
- `bijux-dna run filter`
- `bijux-dna run merge`
- `bijux-dna run contam`
- `bijux-dna run stats-neutral`
- `bijux-dna run umi`
- `bijux-dna run error-correct`
- `bijux-dna run qc`
- `bijux-dna run align`
- `bijux-dna run preprocess`
- `bijux-dna run run`
- `bijux-dna run compare`

Visible aliases are part of the operator surface:

- `bijux-dna run validate` aliases `bijux-dna run validate-pre`.
- `bijux-dna run stats` aliases `bijux-dna run stats-neutral`.

### Pipeline Profiles
- `bijux-dna plan list`
- `bijux-dna plan explain`
- `bijux-dna plan explain-profile`
- `bijux-dna plan validate-profile`
- `bijux-dna plan profile-diff`
- `bijux-dna plan audit`

### Analysis And Explanation
- `bijux-dna analyze runs`
- `bijux-dna analyze summary`
- `bijux-dna analyze compare`
- `bijux-dna analyze rank`
- `bijux-dna analyze report`
- `bijux-dna analyze metrics`
- `bijux-dna analyze bench`
- `bijux-dna explain runs`
- `bijux-dna explain summary`
- `bijux-dna explain compare`
- `bijux-dna explain rank`
- `bijux-dna explain report`
- `bijux-dna explain metrics`
- `bijux-dna explain bench`

### Benchmarking
- `bijux-dna bench config validate`
- `bijux-dna bench run`
- `bijux-dna bench status`
- `bijux-dna bench workspace-value`
- `bijux-dna bench config-json`
- `bijux-dna bench repo-checks`
- `bijux-dna bench write-screen-taxonomy-database-lineage`
- `bijux-dna bench publication-targets`
- `bijux-dna bench corpus-fastq`
- `bijux-dna bench normalize-workspace-layout`
- `bijux-dna bench corpus-fastq-report`
- `bijux-dna bench corpus-fastq-publication-status`
- `bijux-dna bench corpus-fastq-published-dossiers`
- `bijux-dna bench local list-stages`
- `bijux-dna bench local validate-corpus-fixture`
  `validate-corpus-fixture` checks governed corpus fixture manifests such as
  `tests/fixtures/corpora/corpus-01-mini/manifest.toml` and
  `tests/fixtures/corpora/corpus-01-bam-mini/manifest.toml` and
  `tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml` and
  `tests/fixtures/corpora/corpus-03-amplicon-mini/manifest.toml` and
  `tests/fixtures/corpora/corpus-01-adna-damage-mini/manifest.toml` for declared sample identity,
  file-path integrity, source-path provenance, expected taxonomy-output contracts, primer and
  control declarations, amplicon primer-table, expected-ASV, and chimera-control contracts, and
  modality-specific contract checks.
- `bijux-dna bench local validate-corpus-stage-compatibility`
  `validate-corpus-stage-compatibility` checks
  `configs/bench/local/corpus-stage-compatibility.toml` against the governed 51-stage local FASTQ
  and BAM inventories, validates every referenced corpus fixture manifest, and reports which stages
  are covered by corpus-01, corpus-02, corpus-03, or an explicit planner-only reason.
- `bijux-dna bench local validate-pipeline-dag`
  `validate-pipeline-dag` checks governed local pipeline DAG configs such as
  `configs/pipelines/local/fastq-core-preprocess.toml` and
  `configs/pipelines/local/fastq-paired-merge.toml` and
  `configs/pipelines/local/fastq-edna-taxonomy.toml` and
  `configs/pipelines/local/fastq-amplicon.toml` and
  `configs/pipelines/local/fastq-umi.toml` and
  `configs/pipelines/local/bam-core-qc.toml` and
  `configs/pipelines/local/bam-authenticity.toml`, writes a validation report under
  `target/local-ready/pipeline-dag/`, proves the DAG is acyclic, and verifies that every node is
  inventory-aligned with declared inputs, outputs, and dependency handoffs.
- `bijux-dna bench local render-corpus-skip-report`
  `render-corpus-skip-report` writes `target/local-ready/corpus-skip-report.json`, enumerating
  every incompatible corpus-fixture skip with its replacement corpus and keeping planner-only
  stages explicit so no local stage disappears silently.
- `bijux-dna bench local validate-taxonomy-database-fixture`
  `validate-taxonomy-database-fixture` checks governed taxonomy database fixture manifests such as
  `tests/fixtures/databases/taxonomy-mini/manifest.toml` for declared taxa, lineage-table
  consistency, sequence-index paths, classifier-compatibility claims, source-manifest integrity,
  and backend bundle shape.
- `bijux-dna bench local render-benchmark-summary`
  `render-benchmark-summary` writes both `target/local-ready/benchmark-summary.json` and
  `target/local-ready/benchmark-summary.md`, summarizing governed fake-run readiness across all 51
  local FASTQ and BAM benchmark stages.
- `bijux-dna bench local check-manifest-completion`
  `check-manifest-completion` writes `target/local-ready/manifest-completion-report.json` and
  marks a stage complete only when its fake-run `stage-result.json` exists under the selected
  `target/local-fake-runs/stages/` tree.
- `bijux-dna bench local check-output-completion`
  `check-output-completion` writes `target/local-ready/output-completion-report.json` and marks a
  stage complete only when every declared fake-run output exists under the selected
  `target/local-fake-runs/stages/` tree.
- `bijux-dna bench local collect-runtime-metrics`
  `collect-runtime-metrics` writes `target/local-ready/runtime-metrics.json` by reading validated
  fake-run `stage-result.json` manifests and extracting per-stage start, end, elapsed, exit, and
  status fields.
- `bijux-dna bench local render-tool-comparison-template`
  `render-tool-comparison-template` writes
  `target/local-ready/tool-comparison-template.tsv` with one governed row per local benchmark
  stage/tool, carrying runtime, memory, output-metric placeholder, status, and failure-reason
  columns.
- `bijux-dna bench local validate-stage-result`
  `validate-stage-result` loads one `stage-result.json` manifest and fails unless the required
  `command`, `tool`, `runtime`, `resource_metrics`, and `outputs` contract fields are present and
  valid. `resource_metrics.source` must be one of `measured`, `estimated`, or `not_available`.
- `bijux-dna bench local materialize-stage`
- `bijux-dna bench local fake-run-failures`
  `fake-run-failures` writes non-zero stage failure records under
  `target/local-fake-runs/failures/`, including `stderr.txt` and the declared outputs that stayed
  missing for each failed stage.
- `bijux-dna bench local fake-run-stages`
  `fake-run-stages` mirrors every declared benchmark-stage output under
  `target/local-fake-runs/stages/` and writes a fake-run manifest for all governed stages,
  including estimated `resource_metrics` derived from governed thread and memory ceilings.
- `bijux-dna bench local render-stage-commands`
  `render-stage-commands` writes both `target/local-ready/rendered-stage-commands.sh` and the
  machine-readable companion `target/local-ready/rendered-stage-commands.json`.
- `bijux-dna bench schema`
- `bijux-dna bench fastq trim-reads`
- `bijux-dna bench fastq trim-polyg-tails`
- `bijux-dna bench fastq trim-terminal-damage`
- `bijux-dna bench fastq validate-reads`
- `bijux-dna bench fastq detect-adapters`
- `bijux-dna bench fastq profile-read-lengths`
- `bijux-dna bench fastq filter`
- `bijux-dna bench fastq filter-low-complexity`
- `bijux-dna bench fastq merge`
- `bijux-dna bench fastq remove-duplicates`
- `bijux-dna bench fastq remove-chimeras`
- `bijux-dna bench fastq normalize-primers`
- `bijux-dna bench fastq infer-asvs`
- `bijux-dna bench fastq cluster-otus`
- `bijux-dna bench fastq normalize-abundance`
- `bijux-dna bench fastq correct`
- `bijux-dna bench fastq report-qc`
- `bijux-dna bench fastq umi`
- `bijux-dna bench fastq index-reference`
- `bijux-dna bench fastq screen-taxonomy`
- `bijux-dna bench fastq deplete-host`
- `bijux-dna bench fastq deplete-reference-contaminants`
- `bijux-dna bench fastq deplete-rrna`
- `bijux-dna bench fastq profile-reads`
- `bijux-dna bench fastq profile-overrepresented-sequences`
- `bijux-dna bench fastq preprocess`
- `bijux-dna bench bam stage`
- `bijux-dna bench bam pipeline`

Visible FASTQ benchmark aliases are allowed for operator convenience, but canonical docs and tests
should prefer the long names above:

- `trim` for `trim-reads`
- `validate` for `validate-reads`
- `qc-post` for `report-qc`
- `screen` for `screen-taxonomy`
- `stats` for `profile-reads`
- `overrepresented` for `profile-overrepresented-sequences`

## Debug And Repository-Control Commands

These commands are hidden in non-debug builds or exist for repository control-plane work:

- `bijux-dna ena select`
- `bijux-dna ena fetch`
- `bijux-dna tool validate`
- `bijux-dna domain validate`
- `bijux-dna domain coverage`
- `bijux-dna lab corpus list-fastq`
- `bijux-dna config init-hpc`
- `bijux-dna config doctor`
- `bijux-dna config campaign-preflight`
- `bijux-dna config campaign-dry-run`
- `bijux-dna config write-campaign-profiles`
- `bijux-dna slurm submit-stage-benchmark`
- `bijux-dna slurm submit-domain-benchmark`
- `bijux-dna slurm submit-cross-benchmark`
- `bijux-dna slurm submit-campaign`
- `bijux-dna slurm copy-back-manifest`
- `bijux-dna slurm verify-bundle`
- `bijux-dna slurm decrypt-bundle`
- `bijux-dna slurm rewrap-bundle`
- `bijux-dna slurm import-replay`
- `bijux-dna slurm import-campaign`
- `bijux-dna slurm export-failure-bundle`
- `bijux-dna slurm share-bundle`
- `bijux-dna slurm verify-results-policy`
- `bijux-dna bam run`
- `bijux-dna bam list-stages`
- `bijux-dna bam explain`
- `bijux-dna vcf plan`
- `bijux-dna vcf explain`
- `bijux-dna vcf run`
- `bijux-dna validate-manifests`
- `bijux-dna platform`
- `bijux-dna image-qa`
- `bijux-dna replay`
- `bijux-dna compare`
- `bijux-dna policies audit`
- `bijux-dna ci validate`
- `bijux-dna debug`
- `bijux-dna collect`

## Ownership Rules

- Add a command here in the same change that adds a parser variant.
- Keep implementation in the smallest matching `src/commands/*` owner.
- Use `bijux-dna-api` for planning, reporting, domain semantics, and execution contracts.
- Update help snapshots when visible help output changes.
- Prefer canonical command names in docs and tests; list aliases only when they are intentionally
  supported.

## Verification

- Parser changes:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test contracts --no-default-features
```

- Help text changes: update and review `tests/snapshots/*.txt`.
- Command inventory changes:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test boundaries --no-default-features
```
