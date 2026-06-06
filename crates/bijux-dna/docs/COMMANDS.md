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
- `bijux-dna bench validate-matrix`
  `validate-matrix --domain vcf --strict` checks
  `configs/bench/local/vcf-stage-matrix.toml` against the governed VCF stage catalog, the
  production-regression VCF benchmark corpus contract, the VCF required-tool and registry files,
  and the owned adapter, parser, and expected-output contracts. It fails closed when any VCF
  catalog stage is missing, any row drifts from the owned contract set, or any row references a
  tool that is not declared in the VCF benchmark tool inventories.
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
  consistency, per-classifier backend index paths, classifier-compatibility claims, source-manifest
  integrity, and backend bundle shape.
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
