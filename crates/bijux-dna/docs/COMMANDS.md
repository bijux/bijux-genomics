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
- `bijux-dna bench readiness render-adapter-missing-input-tests`
  `render-adapter-missing-input-tests` writes
  `target/bench-readiness/adapter-missing-input-tests.json` with one governed structured failure
  row per benchmark-ready adapter probe, proving that missing FASTQ, BAM, reference, and taxonomy
  database inputs fail early through the real planner entrypoints with the exact missing input role
  preserved in reviewer-visible JSON.
- `bijux-dna bench readiness render-commands`
  `render-commands` writes `target/bench-readiness/rendered-commands.sh` with one governed shell
  command per local benchmark stage command, preserving a parseable `bash` script that can be
  syntax-checked before any HPC-facing submission or wrapper generation.
- `bijux-dna bench readiness render-command-argv`
  `render-command-argv` writes `target/bench-readiness/rendered-commands.argv.jsonl` with one
  governed JSON row per benchmark command, preserving the executable and arguments as a separated
  `argv` array so local benchmark rendering is reproducible without shell-parsing ambiguity.
- `bijux-dna bench readiness render-stage-tool-containers`
  `render-stage-tool-containers` writes `configs/bench/local/stage-tool-containers.toml` with one
  governed row per benchmark-ready FASTQ or BAM stage-tool command, preserving the primary
  execution mode, install kind, declared container identity when available, and the governed
  command entrypoint or explicit host-binary mode needed to keep local and HPC runtime surfaces
  reviewable before submission.
- `bijux-dna bench readiness render-stage-tool-assets`
  `render-stage-tool-assets` writes `configs/bench/local/stage-tool-assets.toml` with one
  governed asset-binding row per benchmark-ready FASTQ or BAM stage-tool command that depends on
  external taxonomy databases, reference indexes, rRNA references, contamination panels,
  haplogroup panels, genotyping sites and regions, or recalibration known-sites inputs, keeping
  the HPC-facing asset contract explicit by `asset_role`, `asset_id`, and `asset_path`.
- `bijux-dna bench readiness render-stage-tool-resources`
  `render-stage-tool-resources` writes `configs/bench/local/stage-tool-resources.toml` with one
  governed row per benchmark-ready FASTQ or BAM stage-tool command, carrying non-zero `threads`,
  `memory_gb`, `walltime_minutes`, and `scratch_gb` hints plus the declared resource-origin
  strategy used to derive those local benchmark defaults.
- `bijux-dna bench readiness render-bam-stage-decision-table`
  `render-bam-stage-decision-table` writes `target/bench-readiness/bam-stage-decision-table.tsv`
  with one governed row per BAM stage in the 24-stage local benchmark slice, classifying each
  stage as `benchmark_ready`, `needs_adapter`, `needs_parser`, `needs_corpus`, or
  `future_not_in_hpc_round` from the current registry, adapter, parser, and corpus surfaces.
- `bijux-dna bench readiness render-bam-command-adapter-coverage`
  `render-bam-command-adapter-coverage` writes
  `target/bench-readiness/bam-command-adapter-coverage.tsv` with one governed row per BAM
  stage-tool binding in the 24-stage benchmark slice, carrying `benchmark_status`,
  `adapter_coverage`, `readiness_gap`, and the underlying `support_status`, `adapter_status`,
  `parser_status`, and `corpus_status`. The report proves which BAM benchmark rows are already
  fully renderable with parser-fixture-validated outputs and fixture-backed corpus coverage while
  keeping parser-blocked, corpus-blocked, and support-blocked rows explicit.
- `bijux-dna bench readiness render-bam-adapter-output-contract`
  `render-bam-adapter-output-contract` writes
  `target/bench-readiness/bam-adapter-output-contract.tsv` with one governed row per BAM
  stage-tool binding in the 24-stage benchmark slice, proving whether each runnable or plannable
  adapter declares every governed stage artifact in both `tool.outputs` and
  `execution_contract.expected_outputs`, identifies the normalized metrics artifact and
  stage-specific raw backend artifacts where applicable, and records the deterministic stdout,
  stderr, and stage-result manifest path templates used by local dry-run execution.
- `bijux-dna bench readiness render-fastq-command-adapter-coverage`
  `render-fastq-command-adapter-coverage` writes
  `target/bench-readiness/fastq-command-adapter-coverage.tsv` with one governed row per FASTQ
  stage-tool binding in the 27-stage benchmark slice, carrying `benchmark_status`,
  `adapter_coverage`, `readiness_gap`, and the underlying `support_status`, `adapter_status`,
  `parser_status`, and `corpus_status`. The report proves which FASTQ benchmark rows are already
  fully renderable with governed support, normalized parsing, and fixture-backed corpus coverage,
  while keeping corpus-blocked and planned-contract rows explicit instead of hidden.
- `bijux-dna bench readiness render-fastq-adapter-output-contract`
  `render-fastq-adapter-output-contract` writes
  `target/bench-readiness/fastq-adapter-output-contract.tsv` with one governed row per FASTQ
  stage-tool binding in the 27-stage benchmark slice, proving whether each runnable or plannable
  adapter declares every governed stage artifact in both `tool.outputs` and
  `execution_contract.expected_outputs`, identifies the normalized metrics artifact and raw output
  artifacts, and records the deterministic stdout, stderr, and stage-result manifest path
  templates used by local dry-run execution.
- `bijux-dna bench readiness render-fastq-tool-serving-map`
  `render-fastq-tool-serving-map` writes `target/bench-readiness/fastq-tool-serving-map.tsv`
  with one governed row per FASTQ stage-tool binding in the 27-stage benchmark slice, carrying
  `tool_id`, `stage_id`, `support_status`, `adapter_status`, `parser_status`, and `corpus_status`
  from the real FASTQ governance contracts and local corpus-compatibility matrix.
- `bijux-dna bench readiness render-bam-tool-serving-map`
  `render-bam-tool-serving-map` writes `target/bench-readiness/bam-tool-serving-map.tsv`
  with one governed row per BAM stage-tool binding in the 24-stage benchmark slice, carrying
  `tool_id`, `stage_id`, `support_status`, `adapter_status`, `parser_status`, and `corpus_status`
  from the real BAM stage catalog, tool contracts, planner admission, and local corpus-compatibility
  matrix. Rows remain visible when a BAM tool binding is governed by stage metadata but lacks a BAM
  tool contract (`missing_contract`) or is admitted by stage metadata but not by the BAM tool YAML
  (`mismatched_contract`).
- `bijux-dna bench readiness render-orphan-tools`
  `render-orphan-tools` writes `target/bench-readiness/orphan-tools.tsv` with one governed row per
  FASTQ or BAM tool contract that exists in scope but serves no currently rendered benchmark stage.
  Each row carries `domain`, `tool_id`, `decision`, `declared_stage_ids`, `benchmark_stage_ids`,
  and `reason`, and every orphan row is forced into an explicit disposition:
  `register_to_stage`, `remove_from_scope`, or `future_tool`.
- `bijux-dna bench readiness render-missing-benchmark-pairs`
  `render-missing-benchmark-pairs` writes
  `target/bench-readiness/missing-benchmark-pairs.tsv` with one governed row per FASTQ or BAM
  stage-tool pair that is admitted by the domain contracts but missing from the current benchmark
  matrix. Each row carries `domain`, `stage_id`, `tool_id`, `support_status`,
  `registered_tool_ids`, and `reason` so compatible pairs cannot disappear silently before
  readiness review.
- `bijux-dna bench readiness render-tool-id-normalization`
  `render-tool-id-normalization` writes
  `target/bench-readiness/tool-id-normalization.tsv` with one governed row per separator-folded
  FASTQ or BAM tool-ID alias cluster, carrying `normalized_tool_id`, `canonical_tool_id`,
  `alias_tool_ids`, `domains`, and `reason` so inconsistent `-` versus `_` benchmark tool naming
  cannot drift without an explicit canonical mapping.
- `bijux-dna bench readiness validate-tool-execution-modes`
  `validate-tool-execution-modes` checks `configs/bench/local/tool-execution-modes.toml` against
  the governed FASTQ and BAM benchmark serving maps plus each tool's runtime probe contract,
  enforcing one primary operator runtime classification for every benchmark tool. The JSON report
  carries `mode_count`, `tool_count`, `multidomain_tool_count`, `mode_counts`, and one row per
  tool with its `execution_mode`, `expected_install_kind`, domains, benchmark stage scope, and
  required runtime fields.
- `bijux-dna bench readiness validate-tool-families`
  `validate-tool-families` checks `configs/bench/local/tool-families.toml` against the governed
  FASTQ and BAM benchmark serving maps, enforcing one primary-function family assignment for every
  benchmark tool. The JSON report carries `family_count`, `tool_count`, `multidomain_tool_count`,
  `family_counts`, and one row per tool with its `family_id`, domains, and governed benchmark
  stage scope.
- `bijux-dna bench readiness render-stage-registry-extra-pairs`
  `render-stage-registry-extra-pairs` writes
  `target/bench-readiness/stage-registry-extra-pairs.tsv` with one governed row per benchmark-
  scoped stage-registry pair that is present in `configs/ci/registry/tool_registry.toml` but not
  admitted by the domain tool contracts. Each row carries `domain`, `stage_id`, `tool_id`,
  `contract_status`, `registry_sources`, `registered_stage_ids`, `intentional_override_status`,
  `intentional_override_reason`, and `reason` so compiled registry scope cannot silently outrun
  domain truth.
- `bijux-dna bench readiness render-unregistered-benchmark-pairs`
  `render-unregistered-benchmark-pairs` writes
  `target/bench-readiness/unregistered-benchmark-pairs.tsv` with one governed row per FASTQ or
  BAM benchmark-matrix pair that is missing from `configs/ci/registry/tool_registry.toml`. Each
  row carries `domain`, `stage_id`, `tool_id`, `support_status`, `registry_status`,
  `registered_stage_ids`, and `reason` so benchmark scope cannot silently outrun the production
  tool registry.
- `bijux-dna bench readiness render-undercovered-stages`
  `render-undercovered-stages` writes `target/bench-readiness/undercovered-stages.tsv` with one
  governed row per benchmark stage that admits multiple tool options in the domain tool contracts
  but currently registers only one tool in the benchmark serving map. Each row carries `domain`,
  `stage_id`, `valid_tool_count`, `registered_tool_count`, `valid_tool_ids`,
  `registered_tool_ids`, `missing_tool_ids`, and `reason` so single-backend benchmark gaps remain
  visible before HPC campaign hardening.
- `bijux-dna bench local list-stages`
- `bijux-dna bench local validate-hpc-submission-ready`
  `validate-hpc-submission-ready` writes `target/local-ready/HPC_SUBMISSION_READY.json` and
  reruns the governed local readiness proof slice end to end: stage matrices, numbered FASTQ and
  BAM local smoke or plan artifacts, benchmark harness fake-run checks, mini corpus fixtures,
  pipeline DAG validations, watchdog simulations, HPC campaign profile dry-runs, and SLURM dry-run
  validation. It fails only after writing the report, and reports the exact failing goal IDs when
  any governed surface regresses.
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
- `bijux-dna bench local judge-taxonomy-output`
  `judge-taxonomy-output` compares governed eDNA taxonomy reports against
  `tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv`, writes
  `target/local-ready/corpus-02-edna-taxonomy-judgment.json` by default, and fails when any
  declared sample is missing an observed report or any expected present or absent taxon does not
  match the observed classifier summary.
- `bijux-dna bench local validate-corpus-stage-compatibility`
  `validate-corpus-stage-compatibility` checks
  `configs/bench/local/corpus-stage-compatibility.toml` against the governed 51-stage local FASTQ
  and BAM inventories, validates every referenced corpus fixture manifest, and reports which stages
  are covered by corpus-01, corpus-02, corpus-03, or an explicit planner-only reason.
- `bijux-dna bench local validate-pipeline-dag`
  `validate-pipeline-dag` checks governed local pipeline DAG configs such as
  `configs/pipelines/local/fastq-core-preprocess.toml` and
  `configs/pipelines/local/fastq-to-bam.toml` and
  `configs/pipelines/local/fastq-paired-merge.toml` and
  `configs/pipelines/local/fastq-edna-taxonomy.toml` and
  `configs/pipelines/local/fastq-amplicon.toml` and
  `configs/pipelines/local/fastq-umi.toml` and
  `configs/pipelines/local/bam-core-qc.toml` and
  `configs/pipelines/local/bam-authenticity.toml` and
  `configs/pipelines/local/bam-genotyping.toml` and
  `configs/pipelines/local/bam-kinship.toml`, writes a validation report under
  `target/local-ready/pipeline-dag/`, proves the DAG is acyclic, and verifies that every node is
  inventory-aligned with declared inputs, outputs, and dependency handoffs, including governed
  mixed FASTQ-to-BAM path handoffs for cross-domain DAGs.
- `bijux-dna bench local simulate-dag-watchdog`
  `simulate-dag-watchdog` writes governed DAG scheduling simulations such as
  `target/local-ready/dag-sim/no-global-wait.json` and
  `target/local-ready/dag-sim/failure-isolation.json` and
  `target/local-ready/dag-sim/partial-resume.json` and
  `target/local-ready/dag-sim/completion-rules.json`, proving that dependency-ready nodes can
  start without a global branch barrier, that one failed sample-stage does not block unrelated
  sample work, that valid completed nodes are reused while only missing or invalid work is
  replanned, and that zero exit status alone does not mark a node complete unless declared outputs
  and the result manifest also exist.
- `bijux-dna bench local render-corpus-skip-report`
  `render-corpus-skip-report` writes `target/local-ready/corpus-skip-report.json`, enumerating
  every incompatible corpus-fixture skip with its replacement corpus and keeping planner-only
  stages explicit so no local stage disappears silently.
- `bijux-dna bench local validate-taxonomy-database-fixture`
  `validate-taxonomy-database-fixture` checks governed taxonomy database fixture manifests such as
  `tests/fixtures/databases/taxonomy-mini/manifest.toml` for declared taxa, lineage-table
  consistency, sequence-index paths, classifier-compatibility claims, source-manifest integrity,
  and backend bundle shape.
- `bijux-dna bench local validate-slurm-dependencies`
  `validate-slurm-dependencies` writes `target/slurm-dry-run/dependency-check.json` and refuses
  any dry-run job whose dependencies are split or duplicated across both the submit manifest and
  the generated `.sbatch` header.
- `bijux-dna bench local validate-slurm-shell-syntax`
  `validate-slurm-shell-syntax` writes `target/slurm-dry-run/bash-n-report.json` and refuses any
  generated `.sbatch` file under the selected dry-run root that fails `bash -n`.
- `bijux-dna bench local validate-slurm-script-bodies`
  `validate-slurm-script-bodies` writes `target/slurm-dry-run/no-placeholder-report.json` and
  refuses generated `.sbatch` bodies that still contain placeholder markers, fake `echo execute`
  payloads, unconditional `rc=0`, or missing `bijux-dna` command lines.
- `bijux-dna bench local render-slurm-submit-manifest`
  `render-slurm-submit-manifest` writes `target/slurm-dry-run/submit-manifest.json`, rendering the
  governed FASTQ and BAM dry-run script slices first and then recording per-job job names, domain,
  stage ownership, corpus and sample scope, resources, script path, log paths, declared outputs,
  and derived dependencies.
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
- `bijux-dna bench local render-slurm-scripts`
  `render-slurm-scripts` writes one governed `.sbatch` file per selected local benchmark stage
  under `target/slurm-dry-run/<domain>/`, using the real stage materialization command,
  governed thread and memory ceilings, and a domain-scoped stage inventory slice such as the
  27-stage FASTQ local benchmark matrix or the 24-stage BAM local benchmark matrix. Each script
  now resolves `#SBATCH --output`, `#SBATCH --error`, `RESULT_ROOT`, and
  `STAGE_RESULT_MANIFEST_PATH` under
  `target/slurm-dry-run/runs/local-benchmark-dry-run/<corpus-or-planner-only>/<stage-id-or-pipeline-id>/<sample-id-or-sample-set>/<tool-id>/`.
  Rendering the BAM slice requires `cargo run -p bijux-dna --features bam_downstream -- bench local
  render-slurm-scripts --domain bam`.
- `bijux-dna bench local render-stage-commands`
  `render-stage-commands` writes both `target/local-ready/rendered-stage-commands.sh` and the
  machine-readable companions `target/local-ready/rendered-stage-commands.json` and
  `target/local-ready/rendered-stage-commands.argv.jsonl`.
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
