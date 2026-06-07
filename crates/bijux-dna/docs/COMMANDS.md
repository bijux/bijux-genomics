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

### Fixtures
- `bijux-dna fixtures build`
  `fixtures build --corpus vcf-mini --out target/local-ready/vcf-mini-regeneration` regenerates
  the owned `vcf-mini` corpus from the repo-side reference, metadata, interval, and variant
  contract, writes the expected-truth bundle and `CHECKSUMS.sha256`, then emits
  `target/local-ready/vcf-mini-regeneration/manifest.json`. It fails closed if the regenerated
  fixture or truth bundle does not validate or if the regenerated sample, population, interval,
  variant, or cohort-pair counts drift from the governed fixture.
- `bijux-dna fixtures validate`
  `fixtures validate --corpus vcf-mini` validates the governed
  `tests/fixtures/corpora/vcf-mini/manifest.toml` contract against the owned reference FASTA and
  FAI, single-sample/cohort/phased/panel VCF assets, target-sites BED, and sample/population
  metadata. It fails closed when any declared file is missing, when the reference and FAI drift,
  or when VCF sample ids fall out of sync with the metadata tables.
- `bijux-dna fixtures validate-expected`
  `fixtures validate-expected --corpus vcf-mini` validates the governed
  `tests/fixtures/corpora/vcf-mini/expected/*.json` truth bundle against the owned multisample,
  phased, panel, filtered, and raw VCF assets plus the cohort metadata contract. It fails closed
  when variant counts, sample missingness, genotype-state tallies, allele frequencies, phasing
  status, pairwise cohort distances, ROH expectations, or IBD expectations drift away from the
  governed fixture corpus.

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
- `bijux-dna plan validate`
- `bijux-dna plan validate-profile`
- `bijux-dna plan profile-diff`
- `bijux-dna plan audit`

Visible aliases are part of the operator surface:

- `bijux-dna pipeline validate` aliases `bijux-dna plan validate`.

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
- `bijux-dna bench validate-matrix`
  `validate-matrix --domain vcf --strict` checks
  `configs/bench/local/vcf-stage-matrix.toml` against the governed VCF stage catalog, the
  production-regression VCF benchmark corpus contract, the VCF required-tool and registry files,
  and the owned adapter, parser, and expected-output contracts. It fails closed when any VCF
  catalog stage is missing, any row drifts from the owned contract set, or any row references a
  tool that is not declared in the VCF benchmark tool inventories.
- `bijux-dna bench validate-schemas`
  `validate-schemas --domain vcf` checks the committed VCF normalized metrics schema files against
  the governed parser-supported VCF stage catalog and writes
  `target/bench-readiness/vcf-schema-validation.json`. The gate fails closed unless the shared
  schema file `schemas/bench/vcf-normalized-metrics.v1.json` and the full
  `schemas/bench/vcf-normalized-metrics/` stage-specific file set match the canonical API-rendered
  contracts exactly, including the required VCF benchmark stages for calling, filtering, QC,
  phasing, imputation, population analyses, ROH, IBD, and demography.
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
- `bijux-dna bench readiness render-parser-failure-tests`
  `render-parser-failure-tests` writes `target/bench-readiness/parser-failure-tests.json` with
  one governed structured row per FASTQ and BAM raw parser failure probe, proving that missing,
  empty, and malformed raw backend outputs are classified explicitly by domain-owned parser
  contracts instead of collapsing into silent zero-metric fallbacks.
- `bijux-dna bench readiness render-parser-completeness-gate`
  `render-parser-completeness-gate` writes `target/bench-readiness/gate-parser-complete.json`
  with one governed row per FASTQ or BAM readiness binding, classifying whether the row belongs
  to benchmark-reporting scope or an excluded cohort and proving that every benchmark-ready row
  is backed by parser fixtures before downstream benchmark reporting proceeds.
- `bijux-dna bench readiness render-expected-benchmark-results`
  `render-expected-benchmark-results` writes
  `target/bench-readiness/expected-benchmark-results.tsv` with one governed row per
  benchmark-ready FASTQ or BAM stage-tool-fixture binding, fixing the expected result root,
  `stage-result.json` path, and output artifact IDs before HPC benchmark submission or
  report-completeness checks proceed.
- `bijux-dna bench readiness render-all-domain-expected-benchmark-results`
  `render-all-domain-expected-benchmark-results` writes
  `target/bench-readiness/expected-benchmark-results-all-domains.tsv` with one governed row per
  benchmark-ready FASTQ, BAM, and VCF result binding. Each row keeps `result_id`, `domain`,
  `stage_id`, `tool_id`, `corpus_id`, `asset_profile_id`, `expected_outputs`,
  `expected_metrics`, and `report_section` explicit, and the command fails closed unless result
  identities stay unique and stable across all governed benchmark-ready domains.
- `bijux-dna bench readiness render-all-domain-harness-ready`
  `render-all-domain-harness-ready` writes
  `target/bench-readiness/ALL_DOMAIN_HARNESS_READY.json` and fail-closes across Goals 278–289.
  The gate reruns the governed all-domain stage inventory, stage-tool table, expected-result
  table, rendered commands, output declarations, fake-runner, fake failures, completion checker,
  parser collector, missing-result behavior, failure classification, and real-smoke subset, then
  reconciles the shared 120-result all-domain harness slice so identity drift cannot pass
  silently between surfaces.
- `bijux-dna bench readiness render-all-domain-failure-classification`
  `render-all-domain-failure-classification` writes
  `target/bench-readiness/failure-classification-all-domains.json` and materializes a governed
  fixture tree under `target/bench-readiness/failure-classification-all-domains-fixture/`. It
  keeps one explicit row for each required failure class across the unified FASTQ, BAM, and VCF
  readiness surface: `missing_input`, `tool_not_found`, `command_failed`, `missing_output`,
  `parser_failed`, `insufficient_data`, and `unsupported_pair`. The command fails closed unless
  every class is triggered by governed evidence instead of collapsing all failures into a generic
  failed status.
- `bijux-dna bench readiness render-all-domain-completion-check`
  `render-all-domain-completion-check` writes
  `target/bench-readiness/completion-check-all-domains.json` and materializes a governed fixture
  tree under `target/bench-readiness/completion-check-all-domains-fixture/`. It seeds five
  distinct incomplete cases across the canonical 120-result FASTQ, BAM, and VCF slice and then
  proves a result is complete only when execution succeeded, the stage manifest is present and
  valid, declared outputs are present, normalized metrics are present, and required run files are
  non-empty.
- `bijux-dna bench readiness render-all-domain-missing-result-test`
  `render-all-domain-missing-result-test` writes
  `target/bench-readiness/missing-result-test-all-domains.json` and materializes a governed
  fixture tree under `target/bench-readiness/missing-result-test-all-domains-fixture/`. It
  removes one governed fake-run manifest from FASTQ, BAM, and VCF, then proves the final table
  still carries exactly three `missing_result` rows instead of dropping those benchmark bindings.
- `bijux-dna bench readiness render-all-domain-output-declarations`
  `render-all-domain-output-declarations` writes
  `target/bench-readiness/output-declarations-all-domains.tsv` with one governed row per
  benchmark-ready FASTQ, BAM, and VCF result binding. Each row keeps raw outputs, normalized
  metrics outputs, logs, result manifest, and index outputs explicit, and the command fails closed
  unless every governed result keeps complete output declarations.
- `bijux-dna bench readiness render-all-domain-parser-collector`
  `render-all-domain-parser-collector` writes
  `target/bench-readiness/parser-collector-all-domains.json` and materializes a governed fixture
  tree under `target/bench-readiness/parser-collector-all-domains-fixture/`. It collects fake-run
  parser evidence for the canonical 120-result FASTQ, BAM, and VCF benchmark-ready slice, joins a
  governed real-smoke subset from each domain, and normalizes both sources into one reviewable
  dataset with stable domain, stage, tool, and manifest identity.
- `bijux-dna bench readiness render-full-benchmark-result-collector`
  `render-full-benchmark-result-collector` writes
  `target/bench-readiness/full-result-collector-test.json`. It merges the canonical FASTQ, BAM,
  and VCF expected-result rows, the 93 essential-pipeline fake-run nodes, the all-domain fake-run
  and fake-failure surfaces, the missing-result audit rows, the governed real-smoke subset, and
  the explicit unsupported-pair row into one reviewable dataset with stable `record_id`,
  `surface_kind`, and `result_status` fields. The command fail-closes unless missing results stay
  distinct from unsupported pairs.
- `bijux-dna bench readiness render-full-benchmark-dashboard`
  `render-full-benchmark-dashboard` writes
  `target/bench-readiness/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.md` and
  `target/bench-readiness/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.json`. It derives the required
  summary counts for total stages, total tools, total expected jobs, ready jobs, blocked jobs,
  missing parsers, missing adapters, missing assets, and failed real smokes directly from the
  governed all-domain stage inventory, expected-result, rendered-command, parser-collector,
  full-report, and real-smoke surfaces. The command fail-closes unless every dashboard number
  matches those machine-readable source surfaces exactly.
- `bijux-dna bench readiness render-full-benchmark-report`
  `render-full-benchmark-report` writes
  `target/bench-readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.md` and
  `target/bench-readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.json`. It renders one canonical report
  row per all-domain expected benchmark binding, keeps the governed missing-result rows visible,
  appends the explicit unsupported pair row, and then builds the required stage-centric,
  tool-centric, corpus-centric, pipeline-centric, runtime, memory, failures, missing-results,
  comparable-metrics, and unsupported-pairs sections from the governed source surfaces. The
  command fail-closes unless the report row count stays equal to the expected-result table count
  plus the explicit unsupported rows.
- `bijux-dna bench readiness render-missing-result-report`
  `render-missing-result-report` writes `target/bench-readiness/missing-result-report-test.json`
  with one governed row per expected FASTQ or BAM benchmark result, materializes a controlled
  fake result tree, removes the governed taxonomy result manifest, and proves that the report
  keeps the missing binding visible as a `missing_result` row instead of dropping it.
- `bijux-dna bench readiness render-pair-readiness`
  `render-pair-readiness` writes `target/bench-readiness/pair-readiness.tsv` with one governed
  row per FASTQ or BAM stage-tool pair, carrying the exact `adapter_status`, `parser_status`,
  `corpus_status`, and `asset_status` columns plus the resolved `readiness_gap` so incomplete
  bindings stay reviewable by the precise missing component instead of collapsing into a generic
  not-ready bucket.
- `bijux-dna bench readiness render-corpus-centric-report`
  `render-corpus-centric-report` writes `target/bench-readiness/corpus-centric-report.md` with
  one governed section per FASTQ or BAM corpus family, carrying the stage inventory that corpus
  exercises plus exact fixture IDs, ready-vs-blocked tool counts, shared metric visibility, and
  blocked tool rows. Taxonomy stays reviewer-visible under `corpus-02`, ASV or OTU or chimera
  stages stay under `corpus-03`, and ancient-DNA, genotyping, kinship, and core BAM analysis
  remain bound to their owned BAM corpora.
- `bijux-dna bench readiness render-benchmark-readiness-dashboard`
  `render-benchmark-readiness-dashboard` writes
  `target/bench-readiness/FASTQ_BAM_BENCHMARK_READINESS.md` and
  `target/bench-readiness/FASTQ_BAM_BENCHMARK_READINESS.json`, aggregating the governed matrix,
  adapter, parser, corpus, asset, and report surfaces into one local dashboard. The summary keeps
  total expected pairs, ready pairs, blocked pairs, exact blocker counts, and every blocked
  `stage_id × tool_id` row reviewer-visible in one place.
- `bijux-dna bench readiness render-stage-tool-benchmark-ready`
  `render-stage-tool-benchmark-ready` writes
  `target/bench-readiness/FASTQ_BAM_STAGE_TOOL_BENCHMARK_READY.json`, proving that the
  benchmark-ready FASTQ/BAM slice is complete enough to generate local HPC benchmark jobs and
  report expectations while keeping every excluded `not_benchmark_ready` pair explicit. The gate
  passes only when the ready slice retains matrix, registry, adapter, parser, corpus, asset,
  expected-result, and report-map coverage; excluded pairs stay visible with exact readiness gaps
  and explicit confirmation that they are omitted from generated jobs and expected results.
- `bijux-dna bench readiness render-tool-centric-report`
  `render-tool-centric-report` writes `target/bench-readiness/tool-centric-report.md` with one
  governed section per benchmarked tool, carrying the full FASTQ/BAM stage list that tool serves
  plus exact `benchmark_status`, `readiness_gap`, `support_status`, `adapter_status`,
  `parser_status`, `corpus_status`, and `asset_status` columns so named tools such as
  `samtools`, `picard`, `fastp`, `vsearch`, `kraken2`, `bowtie2`, and `gatk` stay reviewer-visible
  with complete stage coverage and precise blockers.
- `bijux-dna bench readiness render-stage-centric-report`
  `render-stage-centric-report` writes `target/bench-readiness/stage-centric-report.md` with one
  governed section per FASTQ or BAM benchmark stage, carrying the full tool list benchmarked
  against that stage plus exact `benchmark_status`, `readiness_gap`, `support_status`,
  `adapter_status`, `parser_status`, `corpus_status`, and `asset_status` columns. Multi-tool
  stages keep their shared metric contract visible as `declared`, `not_declared`, or
  `not_applicable`, so stages such as `trim_reads`, `screen_taxonomy`, `index_reference`,
  `damage`, and `contamination` remain reviewer-visible with complete tool coverage and precise
  pending rows.
- `bijux-dna bench readiness render-fastq-report-map`
  `render-fastq-report-map` writes `target/bench-readiness/fastq-report-map.tsv` with one
  governed row per FASTQ benchmark-ready stage, fixing the report section, summary table, and
  benchmark anchor tool that downstream stage-centric benchmark reporting must use.
- `bijux-dna bench readiness render-bam-report-map`
  `render-bam-report-map` writes `target/bench-readiness/bam-report-map.tsv` with one governed
  row per BAM benchmark-ready stage, fixing the report section, summary table, workflow branch,
  and benchmark anchor tool that downstream BAM stage reporting must use.
- `bijux-dna bench readiness render-corpus-asset-coverage-gate`
  `render-corpus-asset-coverage-gate` writes
  `target/bench-readiness/gate-corpus-assets-complete.json` with one governed row per FASTQ or
  BAM readiness binding, classifying whether the row belongs to benchmark-submission scope or an
  excluded cohort and proving that every benchmark-ready row retains governed corpus assignment
  plus any required stage-tool asset bindings before HPC benchmark submission proceeds.
- `bijux-dna bench readiness render-essential-pipeline-corpus-assets`
  `render-essential-pipeline-corpus-assets` writes
  `target/bench-readiness/essential-pipeline-corpus-assets.tsv` with one governed row per node in
  the essential local pipeline set, keeping explicit `pipeline_id`, `node_id`, `stage_id`,
  `corpus_id`, derived `asset_profile_id`, symbolic input handoffs, symbolic output handoffs, and
  resolution status reviewer-visible so no essential pipeline node falls back to implicit global
  corpus or asset paths.
- `bijux-dna bench readiness render-essential-pipeline-partial-resume`
  `render-essential-pipeline-partial-resume` writes
  `target/bench-readiness/essential-pipeline-partial-resume.json` and a governed simulation tree
  under `target/bench-readiness/essential-pipeline-partial-resume-tree/`. It proves partial-resume
  behavior against validated `stage-result.json` manifests, forcing the seeded
  `relatedness-segments-vcf` IBD node to rerun while preserving the independent ROH branch.
- `bijux-dna bench readiness render-essential-pipeline-failure-isolation`
  `render-essential-pipeline-failure-isolation` writes
  `target/bench-readiness/essential-pipeline-failure-isolation.json` and a governed simulation
  tree under `target/bench-readiness/essential-pipeline-failure-isolation-tree/`. It injects a
  real failed `stage-result.json` for the seeded `relatedness-segments-vcf` IBD node, then proves
  that only the dependent demography descendant is blocked while the unrelated ROH branch remains
  completed.
- `bijux-dna bench readiness render-essential-pipeline-report-map`
  `render-essential-pipeline-report-map` writes
  `target/bench-readiness/essential-pipeline-report-map.tsv` with one governed row per declared
  essential-pipeline output symbol, keeping explicit `pipeline_id`, `stage_id`, `tool_id`,
  `output_metric`, `report_section`, and `failure_column`. It fails closed unless every declared
  FASTQ, BAM, and VCF pipeline output is collected into a stable report section.
- `bijux-dna bench readiness render-essential-pipelines-ready`
  `render-essential-pipelines-ready` writes
  `target/bench-readiness/ESSENTIAL_PIPELINES_READY.json` and fail-closes across Goals 261–276. It
  reruns the governed essential pipeline DAG, corpus/assets, command rendering, fake-run,
  partial-resume, failure-isolation, and report-map surfaces, then cross-checks that the shared
  node and output counts still agree.
- `bijux-dna bench readiness render-essential-pipeline-commands`
  `render-essential-pipeline-commands` writes
  `target/bench-readiness/essential-pipelines-rendered-commands.sh` plus
  `target/bench-readiness/essential-pipelines-rendered-commands.argv.jsonl` with one governed row
  per essential pipeline node, preserving real executable command steps for FASTQ, BAM, and VCF
  nodes while keeping owned `bijux-dna` materialization fallbacks explicit for composed local
  stages such as `fastq.report_qc` and feature-gated surfaces such as `bam.genotyping`.
- `bijux-dna bench readiness render-commands`
  `render-commands` writes `target/bench-readiness/rendered-commands.sh` with one governed shell
  command per local benchmark stage command, preserving a parseable `bash` script that can be
  syntax-checked before any HPC-facing submission or wrapper generation.
- `bijux-dna bench readiness render-command-argv`
  `render-command-argv` writes `target/bench-readiness/rendered-commands.argv.jsonl` with one
  governed JSON row per benchmark command, preserving the executable and arguments as a separated
  `argv` array so local benchmark rendering is reproducible without shell-parsing ambiguity.
- `bijux-dna bench readiness render-vcf-commands`
  `render-vcf-commands` writes both `target/bench-readiness/vcf-rendered-commands.sh` and
  `target/bench-readiness/vcf-rendered-commands.argv.jsonl` for the canonical VCF
  `benchmark_ready` slice. The shell script preserves the real multi-step adapter pipelines in a
  `bash -n` parseable form, and the JSONL preserves one governed row per benchmark-ready VCF pair
  with structured `command_steps` argv so VCF command rendering stays executable without shell
  placeholders or synthetic `echo execute` stubs.
- `bijux-dna bench readiness render-all-domain-commands`
  `render-all-domain-commands` writes both
  `target/bench-readiness/rendered-commands-all-domains.sh` and
  `target/bench-readiness/rendered-commands-all-domains.argv.jsonl` with one governed row per
  benchmark-ready FASTQ, BAM, and VCF result binding. Each row keeps the stable `result_id`,
  domain identity, benchmark status, and structured command steps explicit, and the command fails
  closed unless all governed rows render real commands with no placeholder execution text.
- `bijux-dna bench readiness render-vcf-adapters-ready`
  `render-vcf-adapters-ready` writes `target/bench-readiness/VCF_ADAPTERS_READY.json` and
  fail-closes across the governed VCF readiness slice for Goals 231 through 244. The gate reruns
  each owned VCF readiness surface, keeps one explicit row per goal with `goal_id`, `surface`,
  `output_path`, `ok`, and `detail`, and also verifies that the canonical benchmark-ready VCF pair
  set matches across the tool-serving map, executable adapter rows, complete output declarations,
  and rendered shell or argv commands.
- `bijux-dna bench readiness render-stage-tool-containers`
  `render-stage-tool-containers` writes `configs/bench/local/stage-tool-containers.toml` with one
  governed row per benchmark-ready FASTQ or BAM stage-tool command, preserving the primary
  execution mode, install kind, declared container identity when available, and the governed
  command entrypoint or explicit host-binary mode needed to keep local and HPC runtime surfaces
  reviewable before submission.
- `bijux-dna bench readiness render-stage-tool-assets`
  `render-stage-tool-assets` writes `configs/bench/local/stage-tool-assets.toml` with one
  governed asset-binding row per local benchmark FASTQ or BAM stage-tool command that depends on
  external taxonomy databases, database artifact IDs, host or contaminant reference catalogs,
  reference index artifacts, rRNA references, reference-index outputs, contamination panels,
  haplogroup panels, genotyping sites and regions, kinship relatedness panels, or recalibration
  known-sites inputs, keeping the benchmark asset contract explicit by `asset_role`, `asset_id`,
  and `asset_path`. The FASTQ taxonomy slice is only accepted when `centrifuge`, `kaiju`,
  `kraken2`, and `krakenuniq` all keep their governed `taxonomy_database_root` and
  `database_artifact_id` bindings, the `fastq.index_reference` slice is emitted only for the
  canonical local backend selected by the governed plan, and the BAM kinship slice must keep the
  governed `reference_fasta` and `reference_panel` bindings for both `angsd` and `king`.
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
- `bijux-dna bench readiness render-bam-corpus-assignment`
  `render-bam-corpus-assignment` writes `target/bench-readiness/bam-corpus-assignment.tsv` with
  one governed row per admitted BAM stage-tool binding, carrying the resolved
  `corpus_family_id`, `fixture_id`, and the benchmark readiness context that proves whether a row
  belongs on the general FASTQ alignment corpus, the BAM mini corpus, the aDNA BAM fixture, the
  genotyping BAM corpus, or the kinship BAM corpus. Each governed BAM row also carries the owned
  `sample_id`, `input_contract`, `benchmark_limits`, `required_assets`, `expected_outputs`, and
  `skip_behavior` fields sourced from local fixture and config contracts. That keeps ancient-DNA
  damage evidence explicit, makes genotyping rows declare their shared reference, candidate sites,
  target regions, and expected outputs, and makes kinship rows declare their relatedness panel,
  minimum-overlap thresholds, expected outputs, and pairwise skip behavior. The report cross-checks
  the BAM domain-owned routing contract against the local corpus compatibility matrix and the
  governed BAM fixture/config evidence so corpus drift cannot hide behind stage-level labels.
- `bijux-dna bench readiness render-bam-parser-coverage`
  `render-bam-parser-coverage` writes `target/bench-readiness/bam-parser-coverage.tsv` with one
  governed row per BAM stage-tool binding that is already benchmark-ready. Each row carries
  `parser_coverage`, `parser_status`, `support_status`, `adapter_status`, and `corpus_status`,
  proving that the benchmark-ready BAM slice stays fully parser-fixture-validated while still
  reporting any excluded non-benchmark-ready gaps in the JSON summary instead of hiding coverage
  drift behind aggregate percentages.
- `bijux-dna bench readiness render-bam-normalized-metrics-schema`
  `render-bam-normalized-metrics-schema` writes
  `schemas/bench/bam-normalized-metrics.v1.json` with the governed JSON Schema contract for
  normalized BAM parser outputs. The readiness report enumerates every benchmark BAM stage
  extension, its durable extension ID, and the required normalized key count so schema drift
  cannot hide behind backend-specific metric layouts.
- `bijux-dna bench readiness render-bam-comparable-metrics`
  `render-bam-comparable-metrics` writes
  `target/bench-readiness/bam-comparable-metrics.tsv` with one governed row per BAM stage that
  still has more than one admitted comparable tool in the BAM benchmark surface. Each row carries
  the admitted comparable tools, the default tool, the current corpus-routing status, and the
  governed shared metric fields that make same-stage BAM tool comparisons interpretable without
  relying on tool-private report details.
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
- `bijux-dna bench readiness render-fastq-comparable-metrics`
  `render-fastq-comparable-metrics` writes
  `target/bench-readiness/fastq-comparable-metrics.tsv` with one governed row per FASTQ stage
  that both participates in the observer-specialized comparable benchmark surface and still has
  more than one comparable tool. Each row carries the admitted comparable tools, the default tool,
  the current corpus-routing status, and the governed shared metric fields that make same-stage
  tool comparisons interpretable without relying on tool-private report details.
- `bijux-dna bench readiness render-fastq-corpus-assignment`
  `render-fastq-corpus-assignment` writes
  `target/bench-readiness/fastq-corpus-assignment.tsv` with one governed row per FASTQ
  stage-tool binding in the 27-stage benchmark slice, carrying `benchmark_status`,
  `support_status`, `adapter_status`, `parser_status`, and either an assigned
  `corpus_family_id` plus `fixture_id` or a precise `excluded_reason`. The report validates that
  every benchmark-ready FASTQ row maps to the correct corpus family (`corpus-01`, `corpus-02`, or
  `corpus-03`) while keeping planner-only or intentionally excluded corpus gaps explicit in a
  reviewer-stable table, and it refuses taxonomy coverage drift unless `centrifuge`, `kaiju`,
  `kraken2`, and `krakenuniq` all stay on `corpus-02-edna-mini`, and it refuses amplicon drift
  unless `cutadapt`, `dada2`, `vsearch`, `seqkit`, and `seqfu` all stay on
  `corpus-03-amplicon-mini`.
- `bijux-dna bench readiness render-corpus-incompatibility`
  `render-corpus-incompatibility` writes
  `target/bench-readiness/corpus-incompatibility.tsv` with one governed row per benchmark-ready
  FASTQ or BAM stage-tool binding against each incompatible alternative fixture, carrying the
  incompatible fixture IDs, the required governed replacement, the `incompatibility_kind`, any
  required stage assets, and the exact contract evidence that blocks the mismatch. The report is
  sourced from the real corpus-compatibility matrix, the FASTQ and BAM corpus-assignment reports,
  the stage-tool asset contract, and the owned amplicon, taxonomy, and kinship fixture contracts,
  so ASV-on-corpus-01, taxonomy-outside-corpus-02, and kinship-without-pair-manifest failures
  stay reviewer-visible before HPC submission.
- `bijux-dna bench readiness render-fastq-normalized-metrics-schema`
  `render-fastq-normalized-metrics-schema` writes
  `schemas/bench/fastq-normalized-metrics.v1.json` with the governed JSON Schema contract for
  normalized FASTQ parser outputs. The readiness report enumerates every benchmark FASTQ stage
  extension, its durable extension ID, and the required normalized key count so schema drift
  cannot hide behind parser-specific report formats.
- `bijux-dna bench readiness render-fastq-parser-coverage`
  `render-fastq-parser-coverage` writes `target/bench-readiness/fastq-parser-coverage.tsv` with
  one governed row per FASTQ stage-tool binding that already has governed support,
  adapter-backed command rendering, and fixture-backed corpus coverage. Each row carries
  `parser_coverage`, `parser_status`, `support_status`, `adapter_status`, and `corpus_status`,
  proving that the benchmark-ready FASTQ slice still has full normalized parser coverage instead
  of letting parser drift hide inside the broader readiness summary.
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
  from the real FASTQ governance contracts and local corpus-compatibility matrix. The amplicon
  branch is only accepted when primer normalization, chimera removal, ASV inference, OTU
  clustering, and abundance normalization all remain fixture-backed by
  `corpus-03-amplicon-mini`.
- `bijux-dna bench readiness render-bam-tool-serving-map`
  `render-bam-tool-serving-map` writes `target/bench-readiness/bam-tool-serving-map.tsv`
  with one governed row per BAM stage-tool binding in the 24-stage benchmark slice, carrying
  `tool_id`, `stage_id`, `support_status`, `adapter_status`, `parser_status`, and `corpus_status`
  from the real BAM stage catalog, tool contracts, planner admission, and local corpus-compatibility
  matrix. Rows remain visible when a BAM tool binding is governed by stage metadata but lacks a BAM
  tool contract (`missing_contract`) or is admitted by stage metadata but not by the BAM tool YAML
  (`mismatched_contract`).
- `bijux-dna bench readiness render-vcf-tool-serving-map`
  `render-vcf-tool-serving-map` writes `target/bench-readiness/vcf-tool-serving-map.tsv` with one
  governed row per VCF stage-tool matrix binding, carrying `tool_id`, `stage_id`,
  `support_status`, `adapter_status`, `parser_status`, `corpus_status`, `asset_status`, and
  `benchmark_status` from the owned VCF stage catalog and matrix. The report fails closed unless
  every matrix row appears exactly once and the supported-vs-planned split remains aligned with
  the canonical VCF stage contracts.
- `bijux-dna bench readiness render-all-domain-stage-tool-table`
  `render-all-domain-stage-tool-table` writes
  `target/bench-readiness/all-domain-stage-tool-table.tsv` with one governed row per FASTQ, BAM,
  and VCF stage-tool binding in the unified benchmark-ready and planned local surface. Each row
  keeps `domain`, `stage_id`, `tool_id`, `corpus_id`, `asset_profile_id`, `adapter_id`,
  `parser_id`, and `benchmark_status` explicit. FASTQ and BAM rows use normalized owned adapter
  and parser surface ids, while VCF rows preserve the exact matrix-backed adapter and parser ids.
  The command fails closed unless every benchmark-ready row from the domain-local readiness tables
  appears exactly once in the unified cross-domain view.
- `bijux-dna bench readiness render-vcf-comparable-metrics`
  `render-vcf-comparable-metrics` writes `target/bench-readiness/vcf-comparable-metrics.tsv`
  with one governed row per shared normalized metric across the retained multi-tool VCF stage
  slice. Each row keeps `stage_id`, `metric_id`, `metric_name`, `unit`, `direction`, `required`,
  and `tools_covered`, and the command fails closed unless every retained multi-tool stage has at
  least one governed normalized metric shared across all covered tools.
- `bijux-dna bench readiness render-vcf-expected-benchmark-results`
  `render-vcf-expected-benchmark-results` writes
  `target/bench-readiness/vcf-expected-benchmark-results.tsv` with one row per benchmark-ready VCF
  stage-tool-corpus-asset binding. Each row keeps `domain`, `stage_id`, `tool_id`, `corpus_id`,
  `asset_profile_id`, `expected_outputs`, `expected_metrics`, and `report_section`, and the
  command fails closed unless the owned benchmark-ready VCF slice retains complete expected-result
  coverage.
- `bijux-dna bench readiness render-vcf-missing-result-report`
  `render-vcf-missing-result-report` writes
  `target/bench-readiness/vcf-missing-result-report-test.json` after materializing a governed fake
  VCF benchmark-result tree under `target/bench-readiness/vcf-missing-result-report-fixture`,
  deleting exactly one governed VCF manifest, and auditing the remaining rows. The report fails
  closed unless exactly one row is visible as `missing_result` instead of disappearing from the
  table.
- `bijux-dna bench readiness render-vcf-report-map`
  `render-vcf-report-map` writes `target/bench-readiness/vcf-report-map.tsv` with one row per
  expected benchmark-ready VCF result. Each row keeps `stage_id`, `tool_id`, `section_id`,
  `summary_table`, `metric_columns`, and `failure_columns`, and the command fails closed unless
  every expected VCF result row maps to one governed report section and summary table.
- `bijux-dna bench readiness render-vcf-parsers-report-ready`
  `render-vcf-parsers-report-ready` writes `target/bench-readiness/VCF_PARSERS_REPORT_READY.json`
  and fail-closes across Goals 246–260. The gate reruns the governed VCF schema, parser-fixture,
  parser-failure, parser-coverage, expected-result, missing-result, comparable-metric, and
  report-map surfaces, then validates that parser coverage, expected results, and report mapping
  stay aligned on the same benchmark-ready VCF pair slice.
- `bijux-dna bench readiness render-vcf-parser-coverage`
  `render-vcf-parser-coverage` writes `target/bench-readiness/vcf-parser-coverage.tsv` with one
  row per benchmark-ready VCF stage-tool parser surface. Each row keeps `stage_id`, `tool_id`,
  `parser_id`, `fixture_path`, `schema_id`, and `coverage_status`, and the command fails closed
  unless every benchmark-ready VCF row has a governed parser fixture and schema mapping.
- `bijux-dna bench readiness render-vcf-normalized-metrics-schema`
  `render-vcf-normalized-metrics-schema` writes
  `schemas/bench/vcf-normalized-metrics.v1.json` plus one stage-specific schema file under
  `schemas/bench/vcf-normalized-metrics/` for every parser-supported VCF stage. The readiness
  report enumerates each stage schema version, durable schema ID, stage file name, extension ID,
  and required normalized key count so parser-owned VCF metrics stay governed by one shared schema
  family instead of drifting into tool-private JSON shapes.
- `bijux-dna bench readiness render-vcf-parser-failure-tests`
  `render-vcf-parser-failure-tests` writes
  `target/bench-readiness/vcf-parser-failure-tests.json` with seven governed malformed-output
  probes across the retained VCF parser surfaces: empty output, malformed VCF, missing index,
  missing sample column, malformed PCA table, malformed imputation quality JSON, and malformed
  segment TSV. Each passing row must retain `parser_id`, `stage_id`, `tool_id`, and a structured
  `failure_reason`, and the command fails closed if any parser panics or accepts malformed raw
  output.
- `bijux-dna bench readiness render-vcf-adapter-missing-input-tests`
  `render-vcf-adapter-missing-input-tests` writes
  `target/bench-readiness/vcf-adapter-missing-input-tests.json` with one governed missing-input
  probe row for each required Goal 243 VCF role: `bam`, `bai`, `fasta`, `fai`, `vcf`,
  `vcf_index`, `sites_bed`, `panel_vcf`, `map_file`, and `sample_metadata`. The report replays
  adapter-contract validation before any external tool argv can run, and it keeps the one honest
  support-only exception explicit: `sites_bed` is owned by the governed `vcf-mini` fixture
  contract because no retained VCF adapter currently consumes a target-sites BED directly.
- `bijux-dna bench readiness render-vcf-adapter-output-coverage`
  `render-vcf-adapter-output-coverage` writes
  `target/bench-readiness/vcf-adapter-output-coverage.tsv` with one governed row per retained VCF
  adapter binding across `bcftools`, `angsd`, `plink`, `plink2`, `eigensoft`, `shapeit5`,
  `eagle`, `beagle`, `germline`, `ibdseq`, `ibdhap`, and `ibdne`. Each row keeps explicit raw
  outputs, normalized parser-facing outputs, deterministic stdout/stderr/stage-result paths, and
  index outputs where compressed VCF or BCF artifacts require them. The report fails closed unless
  every benchmark-ready VCF row has complete output declarations.
- `bijux-dna bench readiness render-vcf-angsd-adapter`
  `render-vcf-angsd-adapter` writes `target/bench-readiness/adapters/angsd.vcf.json` with one
  governed row per admitted VCF `angsd` registry binding. Each row keeps the materialized BAM-list
  helper when the stage is BAM-backed, the governed reference and sites inputs, the declared
  likelihood model, the output prefix, the parser output ids, the concrete `angsd` argv, and a
  real missing-input probe result, so GL calling, pseudohaploid calling, damage-aware calling, and
  VCF-GL propagation cannot silently fall back to placeholders or drift away from the owned command
  contract.
- `bijux-dna bench readiness render-vcf-descent-family-adapter`
  `render-vcf-descent-family-adapter` writes
  `target/bench-readiness/adapters/descent-family.vcf.json` with one governed row per retained VCF
  descent binding across `plink2` for `vcf.roh`, `germline`, `ibdseq`, and `ibdhap` for `vcf.ibd`,
  and `ibdne` for `vcf.demography`. Each row keeps the cohort VCF or materialized IBD-segment TSV
  input, the normalized ROH, IBD, or demography output, the raw side outputs, the parser output
  ids, the concrete argv, and a real missing-input probe result. The report stays honest about
  benchmark status by keeping `plink2`, `germline`, and `ibdne` benchmark-ready while `ibdseq` and
  `ibdhap` remain explicit retained rows that are still not benchmark-ready.
- `bijux-dna bench readiness render-vcf-eigensoft-adapter`
  `render-vcf-eigensoft-adapter` writes `target/bench-readiness/adapters/eigensoft.vcf.json` with
  one governed row per admitted VCF `eigensoft` registry binding. Each row keeps the concrete
  `convertf` par file, the declared `.geno`, `.snp`, and `.ind` conversion outputs, the
  `smartpca` par file, the declared `.evec`, `.eval`, and `.smartpca.log` outputs, plus the
  normalized mapping to `pca_report` or `population_structure_report`. The report fails closed
  unless both governed EIGENSOFT rows retain real conversion and PCA command rendering instead of
  drifting back to manual conversion or placeholder argv.
- `bijux-dna bench readiness render-vcf-shapeit5-adapter`
  `render-vcf-shapeit5-adapter` writes `target/bench-readiness/adapters/shapeit5.vcf.json` with
  the governed benchmark-ready `vcf.phasing` row for `shapeit5`. The report materializes the owned
  reference panel and genetic map, keeps the input cohort VCF, phased output VCF, index output,
  phase-block and switch-proxy reports, parser-visible phasing QC and phasing manifest outputs,
  log output, concrete `shapeit5 phase_common` argv, and a real missing-input probe result, so the
  retained benchmark phasing backend cannot drift back to placeholders or implicit panel/map
  wiring.
- `bijux-dna bench readiness render-vcf-eagle-adapter`
  `render-vcf-eagle-adapter` writes `target/bench-readiness/adapters/eagle.vcf.json` with the
  retained-but-not-benchmark-ready `vcf.phasing` row for `eagle`. The report keeps the governed
  input cohort VCF, owned reference panel and genetic map, phased VCF, index output, phasing QC
  and phasing manifest parser outputs, log output, concrete `eagle --vcfTarget ... --vcfRef ...
  --geneticMapFile ... --outPrefix ...` argv, and a real missing-input probe result, so retained
  phasing coverage stays reviewer-visible instead of hiding behind the matrix’s single benchmark
  row.
- `bijux-dna bench readiness render-vcf-beagle-adapter`
  `render-vcf-beagle-adapter` writes `target/bench-readiness/adapters/beagle.vcf.json` with the
  retained-but-not-benchmark-ready `vcf.phasing` row for `beagle`. The report keeps the governed
  input cohort VCF, owned reference panel and genetic map, phased VCF, index output, phasing QC
  and phasing manifest parser outputs, log output, concrete `beagle gt=... ref=... map=... out=...`
  argv, and a real missing-input probe result, so the retained phasing backend stays executable and
  parser-complete even when it is not the current benchmark binding.
- `bijux-dna bench readiness render-vcf-imputation-family-adapter`
  `render-vcf-imputation-family-adapter` writes
  `target/bench-readiness/adapters/imputation-family.vcf.json` with one governed row per retained
  VCF imputation binding across `beagle`, `glimpse`, `impute5`, and `minimac4` for both
  `vcf.imputation` and `vcf.impute`. Each row keeps the target cohort VCF, owned reference panel
  VCF and `panel.m3vcf.gz` when required, genetic map or region literal when required, the
  imputed VCF and index outputs, quality and warning outputs, orchestration and imputation
  manifests, log output, concrete argv, parser output ids, and a real missing-input probe result.
  The report stays honest about benchmark status by keeping `beagle` as the only benchmark-ready
  imputation backend while the other retained rows remain explicit and `not_benchmark_ready`.
- `bijux-dna bench readiness render-vcf-plink-adapter`
  `render-vcf-plink-adapter` writes `target/bench-readiness/adapters/plink.vcf.json` with one
  governed row per admitted VCF `plink` registry binding. Each row keeps the concrete cohort
  command argv, declared input artifacts, raw PLINK outputs such as `.imiss`, `.lmiss`, `.frq`,
  `.het`, `.hwe`, `.bed`, `.bim`, `.fam`, and `.log`, plus the normalized metrics mapping to
  `qc_report` or `admixture_report`. The report fails closed unless every row retains a concrete
  missing-input probe result and an explicit normalized-report mapping instead of leaving raw
  PLINK outputs unmapped.
- `bijux-dna bench readiness render-vcf-plink2-adapter`
  `render-vcf-plink2-adapter` writes `target/bench-readiness/adapters/plink2.vcf.json` with one
  governed row per benchmarked VCF `plink2` matrix binding. Each row keeps the concrete command
  argv, declared input artifacts, raw PLINK2 outputs such as `.smiss`, `.vmiss`, `.afreq`,
  `.het`, `.hardy`, `.eigenvec`, `.eigenval`, `.hom`, `.prune.in`, `.prune.out`, and `.log`,
  plus the normalized metrics mapping to `qc_report`, `pca_report`, `population_structure_report`,
  `admixture_report`, or `roh_report`. The `vcf.admixture` row stays honest about the current
  PLINK2 PCA-proxy contract by keeping eigen outputs explicit instead of pretending PLINK2 owns a
  native Q-matrix artifact.
- `bijux-dna bench readiness render-vcf-bcftools-adapter`
  `render-vcf-bcftools-adapter` writes `target/bench-readiness/adapters/bcftools.vcf.json` with
  one governed row per retained VCF `bcftools` matrix binding. Each row keeps the concrete
  command-step argv, declared input artifacts, raw output artifacts, parser output artifacts, and
  a real missing-input probe result, so calling, filtering, GL propagation, postprocess, stats,
  and panel-preparation rows cannot fall back to placeholder execution or silent input drift.
- `bijux-dna bench readiness render-orphan-tools`
  `render-orphan-tools` writes `target/bench-readiness/orphan-tools.tsv` with one governed row per
  FASTQ or BAM tool contract that exists in scope but serves no currently rendered benchmark stage.
  Each row carries `domain`, `tool_id`, `decision`, `declared_stage_ids`, `benchmark_stage_ids`,
  and `reason`, and every orphan row is forced into an explicit disposition:
  `register_to_stage`, `remove_from_scope`, or `future_tool`.
- `bijux-dna bench readiness render-vcf-orphan-tools`
  `render-vcf-orphan-tools` writes `target/bench-readiness/vcf-orphan-tools.tsv` with one governed
  row per VCF tool that is still registered and required in the VCF tool catalogs but serves zero
  current VCF matrix rows. Each row carries `tool_id`, `registered_binary`, `served_stage_count`,
  and `decision`, and the detector fails closed unless every orphan is explicitly
  `future_not_benchmark_ready` or `remove_from_scope`.
- `bijux-dna bench readiness render-vcf-undercovered-stages`
  `render-vcf-undercovered-stages` writes `target/bench-readiness/vcf-undercovered-stages.tsv`
  with one governed row per VCF stage that admits multiple registered tool backends but currently
  benchmarks only one. Each row carries `stage_id`, `valid_tool_classes`, `registered_tools`,
  `missing_tools`, and `decision`, and the detector fails closed unless every undercovered stage is
  explicitly `future_not_benchmark_ready` or `limit_to_specialized_tool`.
- `bijux-dna bench readiness render-vcf-matrix-registry-consistency`
  `render-vcf-matrix-registry-consistency` writes
  `target/bench-readiness/vcf-matrix-registry-consistency.json` and fails closed unless every VCF
  matrix row is admitted by the VCF registry and every benchmark-ready VCF registry pair is present
  in the matrix. The report keeps `matrix_row_unregistered` and
  `benchmark_ready_registry_pair_missing_from_matrix` drift rows explicit when disagreement returns.
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
  `list-stages` keeps the governed local stage inventory visible by domain. Single-domain JSON
  requests such as `--domain fastq`, `--domain bam`, or `--domain vcf` return the domain-local
  stage inventory directly. Multi-domain requests such as `--domain fastq,bam,vcf` write
  `target/bench-readiness/all-domain-stage-list.json` and report separate `domain_counts` plus
  the unified `total_stage_count` so FASTQ, BAM, and VCF coverage cannot silently drift together.
- `bijux-dna bench local render-vcf-stage-catalog`
  `render-vcf-stage-catalog` writes `configs/bench/local/vcf-stage-catalog.toml`, deriving the
  governed VCF stage catalog from the domain downstream order, stage-spec metadata, and VCF IO
  contracts. Each row keeps `stage_id`, `stage_name`, `support_status`, `default_tool_id`,
  `metrics_schema_id`, `input_types`, `output_types`, `required_assets`, `benchmark_category`,
  and `local_smoke_mode` explicit so the local VCF benchmark surface cannot drift from code.
- `bijux-dna bench local render-vcf-stage-matrix`
  `render-vcf-stage-matrix` writes `configs/bench/local/vcf-stage-matrix.toml`, deriving the
  governed VCF benchmark matrix from the checked-in VCF stage catalog, the production-regression
  VCF corpus contract, and the owned VCF adapter, parser, and expected-output contracts. Each row
  keeps `stage_id`, `tool_id`, `corpus_id`, `asset_profile_id`, `adapter_id`, `parser_id`, and
  `expected_outputs` explicit so every catalog stage stays benchmark-addressable through a real
  local contract row.
- `bijux-dna bench local render-vcf-smoke-root`
  `render-vcf-smoke-root` writes `target/local-smoke/vcf/SMOKE_ROOT.json`, deriving the governed
  local VCF smoke root from the owned VCF stage catalog, benchmark matrix, and current `HEAD`
  revision. The manifest keeps `run_id`, `repo_revision`, `corpus_id`, `created_at`, `command`,
  `stage_count`, `tool_pair_count`, and one explicit row per stage-tool pair with deterministic
  `pair_root`, `artifacts_root`, and `result_manifest_path` values so repeated local smoke runs
  cannot drift onto random temp paths.
- `bijux-dna bench local run-vcf-call-smoke`
  `run-vcf-call-smoke` writes `target/local-smoke/vcf.call/bcftools/calls.vcf.gz`,
  `target/local-smoke/vcf.call/bcftools/calls.vcf.gz.tbi`, and
  `target/local-smoke/vcf.call/bcftools/metrics.json` from the governed `vcf.call` matrix row and
  the real `human_like_validation` BAM fixture in `corpus-01-bam-mini`. The command uses the
  retained `bcftools` caller, resolves the alias onto the diploid BAM calling flow, materializes a
  private reference copy under `artifacts/reference`, and writes a `stage-result.json` manifest
  beside the outputs so reviewers can trace the exact input BAM, reference FASTA, sample identity,
  variant counts, and parseability checks for the real local smoke without mutating the governed
  fixture tree.
- `bijux-dna bench local run-vcf-call-diploid-smoke`
  `run-vcf-call-diploid-smoke` writes `target/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz`,
  `target/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz.tbi`, and
  `target/local-smoke/vcf.call_diploid/bcftools/metrics.json` from the governed
  `vcf.call_diploid` matrix row and the real `human_like_validation` BAM fixture in
  `corpus-01-bam-mini`. The command uses the retained `bcftools` caller, materializes a private
  reference copy under `artifacts/reference`, validates that GT-bearing output stays parseable and
  indexed, and writes a `stage-result.json` manifest plus diploid genotype metrics
  (`ploidy`, `called_genotypes`, `heterozygous_count`, `homozygous_ref_count`,
  `homozygous_alt_count`, and `missing_count`) so diploid genotype representation is proved by a
  real local smoke instead of inferred from the generic calling alias.
- `bijux-dna bench local run-vcf-call-gl-smoke`
  `run-vcf-call-gl-smoke` writes `target/local-smoke/vcf.call_gl/bcftools/gl.vcf.gz`,
  `target/local-smoke/vcf.call_gl/bcftools/gl.vcf.gz.tbi`, and
  `target/local-smoke/vcf.call_gl/bcftools/metrics.json` from the governed `vcf.call_gl` matrix
  row and the real `human_like_validation` BAM fixture in `corpus-01-bam-mini`. The command
  materializes a private reference copy under `artifacts/reference`, emits the real retained
  `bcftools` likelihood representation, validates that the output remains indexed and parseable
  without GT dependence, and records explicit likelihood metrics (`likelihood_field`,
  `sites_with_likelihoods`, `samples_with_likelihoods`, `missing_likelihoods`, and `tool_id`) plus
  a `stage-result.json` manifest so GL-bearing local smoke evidence stays reviewer-visible.
- `bijux-dna bench local run-vcf-damage-filter-smoke`
  `run-vcf-damage-filter-smoke` writes
  `target/local-smoke/vcf.damage_filter/bcftools/damage_filtered.vcf.gz`,
  `target/local-smoke/vcf.damage_filter/bcftools/damage_filtered.vcf.gz.tbi`, and
  `target/local-smoke/vcf.damage_filter/bcftools/metrics.json` from the governed
  `vcf.damage_filter` matrix row. The command materializes a deterministic single-sample synthetic
  VCF with PL-bearing genotype likelihoods plus explicit `CT_GA_DAMAGE_RATIO`, `DEAM5P`,
  `DEAM3P`, and `PMD_SCORE` evidence, runs the real retained `bcftools` damage-filter stage,
  copies out the stage summary/counts/warnings manifests, and records exact smoke metrics
  (`input_variants`, `removed_variants`, `retained_variants`, `damage_context_rule`,
  `terminal_context_count`) so the local smoke proves real damage filtering instead of a fixture
  copy.
- `bijux-dna bench local run-vcf-filter-smoke`
  `run-vcf-filter-smoke` writes `target/local-smoke/vcf.filter/bcftools/filtered.vcf.gz`,
  `target/local-smoke/vcf.filter/bcftools/filtered.vcf.gz.tbi`, and
  `target/local-smoke/vcf.filter/bcftools/metrics.json` from the governed `vcf.filter` matrix
  row. The command materializes a deterministic single-sample site-filter fixture with known
  `LOWQUAL`, `LOW_DP`, `LOW_MQ`, and `HIGH_MISSING` rows, runs the real retained `bcftools`
  filter stage with tagged-record retention, keeps the breakdown and explain artifacts visible,
  and records exact smoke metrics (`input_variants`, `pass_variants`, `failed_variants`,
  `filter_ids`, `depth_threshold`, `quality_threshold`, and `missingness_threshold`) so reviewer
  evidence comes from the repo command instead of an implied threshold contract.
- `bijux-dna bench local run-vcf-qc-smoke`
  `run-vcf-qc-smoke` writes `target/local-smoke/vcf.qc/plink2/qc.json`,
  `target/local-smoke/vcf.qc/plink2/qc_summary.json`, and
  `target/local-smoke/vcf.qc/plink2/metrics.json` from the governed `vcf.qc` matrix row. The
  command materializes a deterministic three-sample cohort VCF with one known high-missingness
  sample and one known high-missingness variant, runs the real retained `plink2`-owned QC stage,
  keeps the copied summary, warnings, histogram, and table artifacts visible, and records exact
  smoke metrics (`sample_missingness`, `variant_missingness`, `maf_summary`, `heterozygosity`,
  `excluded_samples`, and `excluded_variants`) so reviewer evidence comes from the repo command
  instead of an implied parser contract.
- `bijux-dna bench local run-vcf-admixture-smoke`
  `run-vcf-admixture-smoke` writes `target/local-smoke/vcf.admixture/plink2/admixture.tsv` and
  `target/local-smoke/vcf.admixture/plink2/admixture.json` from the governed `vcf.admixture`
  matrix row. The command materializes the owned `vcf-mini` multisample cohort plus population
  metadata contracts, runs the retained admixture stage, keeps the source Q-matrix and stage
  manifest visible, and records exact smoke evidence (`sample_id`, `K`, `cluster_1`, `cluster_2`,
  `status`, `execution_mode`, and any `insufficient_data_reason`) so ancestry proportions remain
  reviewer-visible whether the local run used a real tool path or the governed fallback proxy.
- `bijux-dna bench local run-vcf-population-structure-smoke`
  `run-vcf-population-structure-smoke` writes
  `target/local-smoke/vcf.population_structure/plink2/population_structure.json` from the
  governed `vcf.population_structure` matrix row. The command reruns the owned PCA and admixture
  smoke commands first, fails if their persisted reports are missing, runs the retained population
  structure stage on the governed cohort contract, keeps the source stage JSON, pruned variants,
  logs, and consumed upstream reports visible, and records exact review evidence
  (`consumed_pca`, `consumed_admixture`, `sample_groups`, `distance_summary`, and `status`) so
  the final report is grounded in real upstream local-smoke outputs instead of an invented join.
- `bijux-dna bench local run-vcf-ibd-smoke`
  `run-vcf-ibd-smoke` writes `target/local-smoke/vcf.ibd/germline/ibd.tsv` and
  `target/local-smoke/vcf.ibd/germline/ibd.json` from the governed `vcf.ibd` matrix row. The
  command materializes the owned `vcf-mini` multisample cohort plus sample metadata contract, runs
  the retained IBD stage, keeps the source input, segment, merged, filtered, summary, metrics, and
  log artifacts visible, and records exact normalized pair evidence (`sample_a`, `sample_b`,
  `segment_count`, `total_length`, `overlap_marker_count`, and `status`). It also runs a built-in
  sparse-overlap probe that must mark only IBD as `insufficient_marker_overlap` while a direct ROH
  run on the same sparse input still succeeds, so the suite-local block remains reviewer-visible
  instead of collapsing into a generic command failure.
- `bijux-dna bench local run-vcf-demography-smoke`
  `run-vcf-demography-smoke` writes `target/local-smoke/vcf.demography/ibdne/demography.json`
  from the governed `vcf.demography` matrix row. The command reruns the owned IBD smoke first,
  consumes the governed filtered-segment artifact as its real upstream input, runs the retained
  demography stage, keeps the upstream IBD report plus the source Ne trajectory, contract, metrics,
  and logs visible, and records exact normalized demography evidence (`method`, `input_ibd`,
  `time_bins`, `ne_estimates`, `status`, and `insufficient_reason`). It also reruns demography on
  the built-in sparse-overlap IBD probe and requires an explicit `insufficient_data` result instead
  of a stderr-only failure, so missing IBD support stays reviewer-visible as structured output.
- `bijux-dna bench local run-vcf-roh-smoke`
  `run-vcf-roh-smoke` writes `target/local-smoke/vcf.roh/plink2/roh.tsv` and
  `target/local-smoke/vcf.roh/plink2/roh.json` from the governed `vcf.roh` matrix row. The
  command materializes the owned `vcf-mini` multisample cohort plus sample metadata contract, runs
  the retained ROH stage, keeps the source segment table, per-sample summary, source report,
  metrics, and logs visible, and records exact normalized smoke evidence (`sample_id`, `contig`,
  `start`, `end`, `length`, `variant_count`, `segment_count`, and `total_length`) so reviewer
  evidence comes from the repo command instead of a raw PLINK-shaped artifact.
- `bijux-dna bench local run-vcf-pca-smoke`
  `run-vcf-pca-smoke` writes `target/local-smoke/vcf.pca/plink2/pca.tsv` and
  `target/local-smoke/vcf.pca/plink2/pca.json` from the governed `vcf.pca` matrix row. The
  command materializes the owned `vcf-mini` multisample cohort plus metadata contracts, runs the
  retained PCA stage, keeps the source eigen tables and stage manifest visible, and records exact
  smoke evidence (`sample_id`, `pc1`, `pc2`, `eigenvalues`, `variant_count`, `excluded_samples`,
  and `execution_mode`) so sample-to-metadata coverage stays explicit whether the local run used
  `plink2` directly or the governed fallback proxy.
- `bijux-dna bench local run-vcf-stats-smoke`
  `run-vcf-stats-smoke` writes `target/local-smoke/vcf.stats/bcftools/stats.json`,
  `target/local-smoke/vcf.stats/bcftools/bcftools_stats.txt`, and
  `target/local-smoke/vcf.stats/bcftools/metrics.json` from the governed `vcf.stats` matrix row.
  The command materializes a deterministic two-sample cohort VCF with a known 2:1
  transition/transversion mix, runs the real retained `bcftools` stats stage, fails closed unless
  the persisted `stats.json` stays normalized, and records exact smoke metrics (`variant_count`,
  `snp_count`, `indel_count`, `transition_count`, `transversion_count`, `ti_tv`, and
  `sample_count`) so the local smoke proves normalized benchmark facts instead of only surfacing
  raw tool text.
- `bijux-dna bench local run-vcf-gl-propagation-smoke`
  `run-vcf-gl-propagation-smoke` writes
  `target/local-smoke/vcf.gl_propagation/bcftools/propagated.vcf.gz`,
  `target/local-smoke/vcf.gl_propagation/bcftools/propagated.bcf`, and
  `target/local-smoke/vcf.gl_propagation/bcftools/metrics.json` from the governed
  `vcf.gl_propagation` matrix row. The command materializes a deterministic single-sample GL/PL/GP
  input VCF, runs the real retained `bcftools` propagation stage, keeps the normalized VCF, BCF,
  CSI, and stage report visible, and records exact smoke metrics
  (`input_likelihood_fields`, `output_likelihood_fields`, `lost_fields`, `site_count_before`,
  `site_count_after`) so likelihood-field survival is proved by the repo command instead of
  inferred from policy files.
- `bijux-dna bench local run-vcf-prepare-reference-panel-smoke`
  `run-vcf-prepare-reference-panel-smoke` writes
  `target/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz`,
  `target/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz.tbi`, and
  `target/local-smoke/vcf.prepare_reference_panel/bcftools/metrics.json` from the governed
  `vcf.prepare_reference_panel` matrix row. The command materializes a deterministic single-sample
  input VCF plus an unsorted duplicate-bearing raw panel fixture, runs the real retained
  `bcftools` panel-preparation stage, keeps the overlap, chunk, and panel manifests visible, and
  records exact smoke metrics (`input_variants`, `output_variants`, `sample_count`,
  `duplicate_sites_removed`, `normalization_status`, and `index_path`) so reviewer evidence proves
  the output panel is sorted, indexed, normalized, and sample-consistent instead of only copying a
  panel fixture.
- `bijux-dna bench local run-vcf-phasing-smoke`
  `run-vcf-phasing-smoke` writes `target/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz`,
  `target/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz.tbi`, and
  `target/local-smoke/vcf.phasing/shapeit5/metrics.json` from the governed `vcf.phasing` matrix
  row. The command materializes a deterministic two-sample unphased cohort VCF plus the owned
  panel/map lock assets, runs the real retained `shapeit5` phasing stage, keeps the phasing QC,
  manifest, phase-block, switch-proxy, and panel-asset reports visible, and records exact smoke
  metrics (`input_genotypes`, `phased_genotypes`, `unphased_genotypes`, `phase_set_count`, and
  `tool_id`) so reviewer evidence proves phased separators are emitted instead of only inferring
  phasing readiness from the stage catalog.
- `bijux-dna bench local run-vcf-impute-smoke`
  `run-vcf-impute-smoke` writes `target/local-smoke/vcf.impute/beagle/imputed.vcf.gz`,
  `target/local-smoke/vcf.impute/beagle/imputed.vcf.gz.tbi`, and
  `target/local-smoke/vcf.impute/beagle/metrics.json` from the governed `vcf.impute` matrix row.
  The command materializes a deterministic two-sample masked-truth cohort plus the owned
  panel/map lock assets, runs the real retained `beagle` imputation stage, keeps the imputation
  QC, manifest, overlap, warning, acceptance, and panel-mismatch artifacts visible, and records
  exact smoke metrics (`missing_before`, `missing_after`, `imputed_genotypes`,
  `low_confidence_count`, `masked_truth_site_count`, `masked_truth_match_count`, and
  `unresolved_count`) so reviewer evidence proves a known masked genotype is either filled or
  surfaced with an explicit unresolved reason instead of being hidden by wrapper logic.
- `bijux-dna bench local run-vcf-imputation-metrics-smoke`
  `run-vcf-imputation-metrics-smoke` writes
  `target/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json` from the governed
  `vcf.impute` smoke outputs. The command reruns the real local `beagle` imputation smoke, copies
  the persisted QC, metrics, and manifest artifacts into a dedicated reviewer-facing root, and
  records exact quality evidence (`concordance`, `mean_info_score`, `r2_available`,
  `low_confidence_sites`, and `masked_truth_sites`) so missing imputation-quality fields are kept
  explicit in `missing_quality_fields` instead of silently disappearing from the report surface.
- `bijux-dna bench local run-vcf-call-pseudohaploid-smoke`
  `run-vcf-call-pseudohaploid-smoke` writes
  `target/local-smoke/vcf.call_pseudohaploid/bcftools/pseudohaploid.vcf.gz`,
  `target/local-smoke/vcf.call_pseudohaploid/bcftools/pseudohaploid.vcf.gz.tbi`, and
  `target/local-smoke/vcf.call_pseudohaploid/bcftools/metrics.json` from the governed
  `vcf.call_pseudohaploid` matrix row and the real `human_like_validation` BAM fixture in
  `corpus-01-bam-mini`. The command records explicit pseudohaploid site metrics
  (`target_sites`, `covered_sites`, `called_sites`, `missing_sites`), the governed sampling
  policy, and a declared control seed, then proves deterministic behavior by replaying the real
  stage and comparing a canonicalized VCF payload that strips volatile `bcftools` command-path
  headers while still surfacing the raw header drift.
- `bijux-dna bench local validate-vcf-no-empty-output`
  `validate-vcf-no-empty-output` writes `target/local-ready/vcf/no-empty-output-check.json`,
  refreshes the governed VCF smoke-output fixture tree under `target/local-smoke/vcf`, and fails
  closed unless every declared `.vcf.gz`, `.json`, `.tsv`, and `.log` artifact remains present
  and non-empty. The report keeps one explicit row per checked output with `stage_id`, `tool_id`,
  `output_id`, `output_kind`, `output_path`, `bytes`, `status`, and `allow_empty_reason`, and the
  optional `--skip-refresh` mode lets reviewers prove that a zero-byte artifact is rejected
  instead of being silently regenerated.
- `bijux-dna bench local validate-vcf-stage-catalog-ready`
  `validate-vcf-stage-catalog-ready` writes `target/local-ready/VCF_STAGE_CATALOG_READY.json` and
  fail-closes across the governed VCF Goal 201-209 slice. The gate reruns the owned stage-catalog,
  matrix, corpus-fixture, expected-truth, regeneration, reference-compatibility, sample-compatibility,
  smoke-root, and no-empty-output surfaces, then records one explicit row per goal with
  `goal_id`, `surface`, `output_path`, `ok`, and `detail` so any drift stays visible in a single
  reviewer-facing local-ready report.
- `bijux-dna bench local validate-vcf-smoke-suite-ready`
  `validate-vcf-smoke-suite-ready` writes `target/local-smoke/VCF_SMOKE_SUITE_READY.json` and
  fail-closes across the governed VCF Goal 211-229 slice. The gate reruns every owned VCF local
  smoke surface in goal order, from `vcf.call` through `vcf.demography`, and records one explicit
  row per goal with `goal_id`, `surface`, `output_path`, `ok`, and `detail` so missing outputs,
  unparsable artifacts, or missing normalized evidence stay visible in a single reviewer-facing
  smoke-suite report instead of being hidden in scattered command stderr.
- `bijux-dna bench local validate-vcf-reference-compatibility`
  `validate-vcf-reference-compatibility` writes
  `target/local-ready/vcf/reference-compatibility.json`, deriving the governed VCF contig
  compatibility report from the owned `vcf-mini` fixture manifest, FASTA, FAI, reference
  dictionary, and all declared VCF variant views. The report keeps `reference_id`, `fasta_path`,
  `fai_path`, `dict_path`, `contig_count`, `reference_contigs`, `vcf_contigs`, `missing_contigs`,
  `extra_contigs`, and per-variant-set contig slices explicit, and the command fails closed when
  any VCF contig drifts away from the governed reference contract.
- `bijux-dna bench local validate-vcf-sample-compatibility`
  `validate-vcf-sample-compatibility` writes
  `target/local-ready/vcf/sample-compatibility.json`, deriving the governed cohort-sample
  compatibility report from the owned `vcf-mini` fixture manifest, the multisample and phased VCF
  views, the sample metadata manifest, and the population metadata labels. The report keeps
  `vcf_samples`, `metadata_samples`, `missing_metadata`, `extra_metadata`, `population_labels`,
  `sex_labels`, and explicit missing-label slices visible, and the command fails closed when the
  population-analysis sample set would proceed without known metadata labels.
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
  `configs/pipelines/local/adna-gl-fastq-bam-vcf.toml` and
  `configs/pipelines/local/adna-pseudohaploid-fastq-bam-vcf.toml` and
  `configs/pipelines/local/amplicon-asv-otu-no-vcf.toml` and
  `configs/pipelines/local/bam-genotyping-to-vcf-downstream.toml` and
  `configs/pipelines/local/diploid-small-fastq-bam-vcf.toml` and
  `configs/pipelines/local/popgen-structure-vcf.toml` and
  `configs/pipelines/local/relatedness-segments-vcf.toml` and
  `configs/pipelines/local/reference-panel-imputation.toml` and
  `configs/pipelines/local/fastq-core-preprocess.toml` and
  `configs/pipelines/local/fastq-to-bam.toml` and
  `configs/pipelines/local/core-germline-fastq-bam-vcf.toml` and
  `configs/pipelines/local/fastq-paired-merge.toml` and
  `configs/pipelines/local/edna-taxonomy-no-vcf.toml` and
  `configs/pipelines/local/fastq-edna-taxonomy.toml` and
  `configs/pipelines/local/fastq-amplicon.toml` and
  `configs/pipelines/local/fastq-umi.toml` and
  `configs/pipelines/local/bam-core-qc.toml` and
  `configs/pipelines/local/bam-authenticity.toml` and
  `configs/pipelines/local/bam-genotyping.toml` and
  `configs/pipelines/local/bam-kinship.toml`, writes a validation report under
  `target/local-ready/pipeline-dag/`, proves the DAG is acyclic, and verifies that every node is
  inventory-aligned with declared inputs, outputs, and dependency handoffs, including governed
  mixed FASTQ-to-BAM-to-VCF path handoffs for cross-domain DAGs.
- `bijux-dna plan validate`
  `plan validate --id core-germline-fastq-bam-vcf --strict` resolves the governed local pipeline
  config at `configs/pipelines/local/core-germline-fastq-bam-vcf.toml`, writes
  `target/local-ready/pipeline-dag/core-germline-fastq-bam-vcf.json`, and fails closed unless the
  requested id matches the config identity and the DAG validates with explicit FASTQ, BAM, and VCF
  handoff coverage.
  `plan validate --id adna-pseudohaploid-fastq-bam-vcf --strict` resolves the governed ancient-DNA
  pseudohaploid pipeline config, writes
  `target/local-ready/pipeline-dag/adna-pseudohaploid-fastq-bam-vcf.json`, and fails closed unless
  the validator confirms the terminal-damage trim, BAM damage and authenticity branch, and
  pseudohaploid plus damage-filter VCF path stay explicit.
  `plan validate --id adna-gl-fastq-bam-vcf --strict` resolves the governed ancient-DNA
  genotype-likelihood pipeline config, writes
  `target/local-ready/pipeline-dag/adna-gl-fastq-bam-vcf.json`, and fails closed unless the
  validator confirms the terminal-damage trim, genotype-likelihood call, GL propagation, and VCF QC
  path stay explicitly likelihood-bearing end to end.
  `plan validate --id diploid-small-fastq-bam-vcf --strict` resolves the governed small-sample
  diploid pipeline config, writes
  `target/local-ready/pipeline-dag/diploid-small-fastq-bam-vcf.json`, and fails closed unless the
  validator confirms both the filtered-BAM fallback path and recalibrated-BAM run path remain
  explicit while VCF QC stays independent of optional phasing.
  `plan validate --id reference-panel-imputation --strict` resolves the governed VCF panel-backed
  imputation pipeline config, writes
  `target/local-ready/pipeline-dag/reference-panel-imputation.json`, and fails closed unless panel
  preparation, target QC, optional prephasing, imputation, and downstream imputation metrics keep
  explicit panel identity plus map and reference contracts.
  `plan validate --id popgen-structure-vcf --strict` resolves the governed VCF population-structure
  pipeline config, writes `target/local-ready/pipeline-dag/popgen-structure-vcf.json`, and fails
  closed unless PCA, admixture, and population-structure summary all keep sample metadata and
  population labels as mandatory inputs with explicit metadata-join handoffs.
  `plan validate --id relatedness-segments-vcf --strict` resolves the governed VCF relatedness and
  segment pipeline config, writes `target/local-ready/pipeline-dag/relatedness-segments-vcf.json`,
  and fails closed unless IBD insufficiency remains local to demography instead of blocking ROH or
  QC outputs.
  `plan validate --id bam-genotyping-to-vcf-downstream --strict` resolves the governed BAM-to-VCF
  bridge pipeline config, writes `target/local-ready/pipeline-dag/bam-genotyping-to-vcf-downstream.json`,
  and fails closed unless `vcf.filter` consumes the exact `bam.genotyping` VCF handoff instead of
  a conceptual external cohort-VCF placeholder.
  `plan validate --id edna-taxonomy-no-vcf --strict` resolves the governed eDNA taxonomy-only
  pipeline config, writes `target/local-ready/pipeline-dag/edna-taxonomy-no-vcf.json`, and fails
  closed unless taxonomy screening remains local to FASTQ filtering and reporting instead of
  bridging into BAM or VCF germline stages.
  `plan validate --id amplicon-asv-otu-no-vcf --strict` resolves the governed amplicon-only
  pipeline config, writes `target/local-ready/pipeline-dag/amplicon-asv-otu-no-vcf.json`, and
  fails closed unless ASV, OTU, and abundance outputs remain local to FASTQ reporting instead of
  bridging into BAM or VCF germline stages.
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
  consistency, per-classifier backend index paths, classifier-compatibility claims, source-manifest
  integrity, and backend bundle shape.
- `bijux-dna bench local validate-slurm-dependencies`
  `validate-slurm-dependencies` writes `target/slurm-dry-run/dependency-check.json` and refuses
  any dry-run job whose dependencies are split or duplicated across both the submit manifest and
  the generated `.sbatch` header.
- `bijux-dna bench local validate-slurm-shell-syntax`
  `validate-slurm-shell-syntax` writes `target/slurm-dry-run/bash-n-report.json` and refuses any
  generated `.sbatch` file under the selected dry-run root that fails `bash -n`.
- `bijux-dna bench local validate-all-domain-slurm-shell-syntax`
  `validate-all-domain-slurm-shell-syntax` regenerates the governed all-domain SLURM tree under
  `target/slurm-dry-run/all-domains/`, then writes
  `target/slurm-dry-run/all-domains/bash-n-report.json`. The report refuses any generated
  benchmark-result or essential-pipeline `.sbatch` file in that 213-script tree that fails
  `bash -n`, so the all-domain SLURM surface is syntax-checked as one owned unit instead of
  relying on partial domain roots.
- `bijux-dna bench local validate-slurm-script-bodies`
  `validate-slurm-script-bodies` writes `target/slurm-dry-run/no-placeholder-report.json` and
  refuses generated `.sbatch` bodies that still contain placeholder markers, fake `echo execute`
  payloads, unconditional `rc=0`, or missing `bijux-dna` command lines.
- `bijux-dna bench local render-slurm-submit-manifest`
  `render-slurm-submit-manifest` writes `target/slurm-dry-run/submit-manifest.json`, rendering the
  governed FASTQ and BAM dry-run script slices first and then recording per-job job names, domain,
  stage ownership, corpus and sample scope, resources, script path, log paths, declared outputs,
  and derived dependencies.
- `bijux-dna bench local render-all-domain-slurm-submit-manifest`
  `render-all-domain-slurm-submit-manifest` writes
  `target/slurm-dry-run/all-domains/submit-manifest.json` for the governed 213-job all-domain
  SLURM tree. The manifest keeps one row per benchmark-result or essential-pipeline job with
  `job_id_local`, domain, stage, pipeline or node identity where applicable, tool, corpus,
  asset-profile, script path, stdout, stderr, declared outputs, manifest-only dependencies, and
  resource ceilings. The command fails closed if any dependency points at a non-existent local job
  id or if a generated script leaks dependency ordering into `#SBATCH --dependency` headers.
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
- `bijux-dna bench local run-real-smoke-core-subset`
  `run-real-smoke-core-subset` writes
  `target/local-real-smoke/core-subset/REAL_SMOKE_SUMMARY.json` and records one governed real
  FASTQ smoke stage, one governed real BAM smoke stage, one governed real `vcf.stats` smoke
  stage, and one governed BAM-to-VCF bridge execution through `vcf.call`. Each row keeps the
  parsed evidence path, normalized metrics, and validated `stage-result.json` identity where a
  manifest-backed real execution exists, so the all-domain harness keeps at least one non-fake
  execution slice grounded in real outputs.
- `bijux-dna bench local fake-run-all-domains`
  `fake-run-all-domains` writes one governed fake-run tree under
  `target/local-fake-runs/all-domains/` for every benchmark-ready FASTQ, BAM, and VCF result
  binding in the canonical 120-row all-domain slice. Each result keeps a real `command.sh`,
  `stdout.txt`, `stderr.txt`, `metrics.json`, and `stage-result.json`, plus materialized fake
  declared outputs under `declared-outputs/`, so unified expected-result, command, and
  output-declaration contracts can be exercised without stopping at report generation alone.
- `bijux-dna bench local fake-run-all-domain-failures`
  `fake-run-all-domain-failures` writes one governed failure tree under
  `target/local-fake-runs/all-domains-failures/` for every benchmark-ready FASTQ, BAM, and VCF
  result binding in the canonical 120-row all-domain slice. Each result keeps a real
  `command.sh`, `stderr.txt`, and `failure.json`, and the failure record enumerates the exact
  declared outputs that remained missing, so unified failure handling stays explicit instead of
  collapsing into raw stderr alone.
- `bijux-dna bench local render-all-domain-slurm-scripts`
  `render-all-domain-slurm-scripts` writes one governed `.sbatch` file per canonical all-domain
  benchmark result plus one per essential pipeline node under `target/slurm-dry-run/all-domains/`.
  The generated tree keeps benchmark-ready FASTQ, BAM, and VCF jobs under
  `benchmark-results/<domain>/<corpus>/<stage>/<asset-profile>/<tool>/job.sbatch` and essential
  pipeline jobs under `essential-pipelines/<pipeline-id>/<node-id>/job.sbatch`, while `stdout`,
  `stderr`, and declared outputs resolve under the canonical run root
  `target/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/`. This is the owned
  cross-domain SLURM generation surface for the governed 120-result benchmark slice and the
  93-node essential pipeline slice.
- `bijux-dna bench local validate-all-domain-slurm-script-bodies`
  `validate-all-domain-slurm-script-bodies` regenerates the governed all-domain SLURM tree under
  `target/slurm-dry-run/all-domains/`, then writes
  `target/slurm-dry-run/all-domains/no-placeholder-report.json`. The report fails closed if any
  generated `.sbatch` body contains `placeholder`, `TODO`, `echo execute`, unconditional `rc=0`,
  an empty executable body, or a missing `bijux-dna` invocation, so reviewers can prove the
  all-domain SLURM surface calls owned repo commands instead of template text.
- `bijux-dna bench local validate-all-domain-slurm-result-paths`
  `validate-all-domain-slurm-result-paths` regenerates the governed all-domain submit manifest and
  writes `target/slurm-dry-run/all-domains/path-convention-check.json`. The report fails closed
  unless every benchmark-result and essential-pipeline `stdout`, `stderr`, and declared output
  path lives under the canonical run root
  `target/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/` and keeps stable path
  ownership by domain, stage or pipeline node, tool, corpus, and sample scope where applicable.
- `bijux-dna bench local execute-all-domain-benchmark-result`
  `execute-all-domain-benchmark-result` resolves one canonical all-domain benchmark-ready
  `result_id` back to the owned rendered-command collector and executes the governed command list
  from the repository root. The command exists so generated all-domain `.sbatch` jobs dispatch
  through a stable `bijux-dna` operator surface instead of embedding raw adapter shell fragments.
- `bijux-dna bench local execute-essential-pipeline-node`
  `execute-essential-pipeline-node` resolves one governed essential-pipeline `pipeline_id` /
  `node_id` pair back to the owned rendered-command collector and executes the rendered command
  list from the repository root. The command is the stable execution target for essential-pipeline
  `.sbatch` jobs in the all-domain SLURM tree.
- `bijux-dna bench local fake-run-essential-pipelines`
  `fake-run-essential-pipelines` writes one governed fake-run tree under
  `target/local-fake-runs/pipelines/essential/` for every node in the essential ten-pipeline
  slice. Each node keeps a real `command.sh`, `stdout.txt`, `stderr.txt`, `metrics.json`, and
  `stage-result.json`, plus materialized fake outputs under `declared-outputs/`, so essential
  pipeline validation does not stop at DAG structure or rendered commands.
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
