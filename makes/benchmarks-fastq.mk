##@ Performance Benchmarks

BIJUX_BIN ?= cargo run -q -p bijux-dna-dev -- tooling run bijux --
OUT_DIR ?= .
TOOLS ?=
SAMPLE_ID ?=
R1 ?=
R2 ?=
ALLOW_EXPERIMENTAL ?= 0
PLATFORM ?=
CORPUS_ROOT ?= $(shell python3 makes/bin/benchmark_workspace_value.py remote.corpus_root)

BENCH_TOOLS_ARGS = $(if $(TOOLS),--tools $(TOOLS),)
BENCH_EXPERIMENTAL_ARGS = $(if $(filter 1 true yes,$(ALLOW_EXPERIMENTAL)),--allow-experimental,)

_benchmark-fastq-stage: ## Benchmark FASTQ stage via CLI (requires STAGE=<stage> SAMPLE_ID R1, optional R2)
	@BIJUX_BIN="$(BIJUX_BIN)" BIJUX_PLATFORM="$(PLATFORM)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" STAGE="$(STAGE)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" cargo run -q -p bijux-dna-dev -- tooling run benchmarks fastq-stage

_benchmark-trim: ## Benchmark adapter/quality trimming tools
	@$(MAKE) _benchmark-fastq-stage STAGE=trim SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-validate: ## Benchmark read validation tools
	@$(MAKE) _benchmark-fastq-stage STAGE=validate SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-filter: ## Benchmark contaminant filtering tools
	@$(MAKE) _benchmark-fastq-stage STAGE=filter SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-merge: ## Benchmark read merging tools (paired-end)
	@$(MAKE) _benchmark-fastq-stage STAGE=merge SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-correct: ## Benchmark error correction tools (paired-end)
	@$(MAKE) _benchmark-fastq-stage STAGE=correct SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-qc-post: ## Benchmark post-processing QC tools
	@$(MAKE) _benchmark-fastq-stage STAGE=qc-post SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-umi: ## Benchmark UMI processing tools (paired-end)
	@$(MAKE) _benchmark-fastq-stage STAGE=umi SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-stats: ## Benchmark statistics computation tools
	@$(MAKE) _benchmark-fastq-stage STAGE=stats SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-screen: ## Benchmark screening tools
	@$(MAKE) _benchmark-fastq-stage STAGE=screen SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-preprocess: ## Benchmark full preprocessing pipeline
	@BIJUX_BIN="$(BIJUX_BIN)" BIJUX_PLATFORM="$(PLATFORM)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" cargo run -q -p bijux-dna-dev -- tooling run benchmarks fastq-preprocess

_benchmark-all: ## Run all FASTQ benchmarks sequentially for one explicit sample input
	@BIJUX_BIN="$(BIJUX_BIN)" BIJUX_PLATFORM="$(PLATFORM)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" cargo run -q -p bijux-dna-dev -- tooling run benchmarks fastq-all

_benchmark-status: ## Show canonical benchmark suite/config directories and detected suites
	@BIJUX_BIN="$(BIJUX_BIN)" BIJUX_PLATFORM="$(PLATFORM)" cargo run -q -p bijux-dna-dev -- tooling run benchmarks fastq-status

_benchmark-validate-corpus-01: ## Benchmark fastq.validate_reads across corpus-01
	@python3 makes/bin/run_fastq_validate_reads_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-trim-polyg-corpus-01: ## Benchmark fastq.trim_polyg_tails across corpus-01
	@python3 makes/bin/run_fastq_trim_polyg_tails_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-trim-reads-corpus-01: ## Benchmark fastq.trim_reads across corpus-01
	@python3 makes/bin/run_fastq_trim_reads_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-trim-terminal-damage-corpus-01: ## Benchmark fastq.trim_terminal_damage across corpus-01
	@python3 makes/bin/run_fastq_trim_terminal_damage_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-detect-adapters-corpus-01: ## Benchmark fastq.detect_adapters across corpus-01
	@python3 makes/bin/run_fastq_detect_adapters_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-profile-reads-corpus-01: ## Benchmark fastq.profile_reads across corpus-01
	@python3 makes/bin/run_fastq_profile_reads_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-profile-read-lengths-corpus-01: ## Benchmark fastq.profile_read_lengths across corpus-01
	@python3 makes/bin/run_fastq_profile_read_lengths_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-profile-overrepresented-corpus-01: ## Benchmark fastq.profile_overrepresented_sequences across corpus-01
	@python3 makes/bin/run_fastq_profile_overrepresented_sequences_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-filter-low-complexity-corpus-01: ## Benchmark fastq.filter_low_complexity across corpus-01
	@python3 makes/bin/run_fastq_filter_low_complexity_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-filter-reads-corpus-01: ## Benchmark fastq.filter_reads across corpus-01
	@python3 makes/bin/run_fastq_filter_reads_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-remove-duplicates-corpus-01: ## Benchmark fastq.remove_duplicates across the paired corpus-01 cohort
	@python3 makes/bin/run_fastq_remove_duplicates_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-merge-corpus-01: ## Benchmark fastq.merge_pairs across the paired corpus-01 cohort
	@python3 makes/bin/run_fastq_merge_pairs_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-report-qc-corpus-01: ## Benchmark fastq.report_qc across corpus-01
	@python3 makes/bin/run_fastq_report_qc_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-normalize-primers-corpus-01: ## Benchmark fastq.normalize_primers across corpus-01
	@python3 makes/bin/run_fastq_normalize_primers_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-deplete-rrna-corpus-01: ## Benchmark fastq.deplete_rrna across corpus-01
	@python3 makes/bin/run_fastq_deplete_rrna_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",) \
		$(if $(RRNA_DB),--rrna-db "$(RRNA_DB)",) \
		$(if $(RRNA_BUNDLE_ID),--rrna-bundle-id "$(RRNA_BUNDLE_ID)",)

_benchmark-deplete-host-corpus-01: ## Benchmark fastq.deplete_host across corpus-01
	@python3 makes/bin/run_fastq_deplete_host_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",) \
		$(if $(REFERENCE_INDEX),--reference-index "$(REFERENCE_INDEX)",)

_benchmark-deplete-reference-contaminants-corpus-01: ## Benchmark fastq.deplete_reference_contaminants across corpus-01
	@python3 makes/bin/run_fastq_deplete_reference_contaminants_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",) \
		$(if $(REFERENCE_INDEX),--reference-index "$(REFERENCE_INDEX)",)

_benchmark-screen-taxonomy-corpus-01: ## Benchmark fastq.screen_taxonomy across corpus-01
	@python3 makes/bin/run_fastq_screen_taxonomy_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",) \
		$(if $(DATABASE_ROOT),--database-root "$(DATABASE_ROOT)",)

_benchmark-correct-errors-corpus-01: ## Benchmark fastq.correct_errors across corpus-01
	@python3 makes/bin/run_fastq_correct_errors_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-extract-umis-corpus-01: ## Benchmark fastq.extract_umis across the paired corpus-01 cohort
	@python3 makes/bin/run_fastq_extract_umis_corpus_01.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--out-root "$(OUT_DIR)",) \
		$(if $(PLATFORM),--platform "$(PLATFORM)",) \
		$(if $(TOOLS),--tools "$(TOOLS)",)

_benchmark-validate-corpus-01-report: ## Render the corpus-01 validate benchmark dossier into docs/
	@python3 makes/bin/render_fastq_validate_reads_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_validate_reads_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.validate_reads/corpus-01

_benchmark-trim-polyg-corpus-01-report: ## Render the corpus-01 trim-polyg benchmark dossier into docs/
	@python3 makes/bin/render_fastq_trim_polyg_tails_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_trim_polyg_tails_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.trim_polyg_tails/corpus-01

_benchmark-trim-reads-corpus-01-report: ## Render the corpus-01 trim-reads benchmark dossier into docs/
	@python3 makes/bin/render_fastq_trim_reads_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_trim_reads_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.trim_reads/corpus-01

_benchmark-trim-terminal-damage-corpus-01-report: ## Render the corpus-01 terminal-damage benchmark dossier into docs/
	@python3 makes/bin/render_fastq_trim_terminal_damage_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_trim_terminal_damage_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.trim_terminal_damage/corpus-01

_benchmark-detect-adapters-corpus-01-report: ## Render the corpus-01 detect-adapters benchmark dossier into docs/
	@python3 makes/bin/render_fastq_detect_adapters_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_detect_adapters_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.detect_adapters/corpus-01

_benchmark-profile-reads-corpus-01-report: ## Render the corpus-01 profile-reads benchmark dossier into docs/
	@python3 makes/bin/render_fastq_profile_reads_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_profile_reads_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.profile_reads/corpus-01

_benchmark-profile-read-lengths-corpus-01-report: ## Render the corpus-01 read-length benchmark dossier into docs/
	@python3 makes/bin/render_fastq_profile_read_lengths_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_profile_read_lengths_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.profile_read_lengths/corpus-01

_benchmark-profile-overrepresented-corpus-01-report: ## Render the corpus-01 overrepresented benchmark dossier into docs/
	@python3 makes/bin/render_fastq_profile_overrepresented_sequences_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_profile_overrepresented_sequences_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.profile_overrepresented_sequences/corpus-01

_benchmark-filter-low-complexity-corpus-01-report: ## Render the corpus-01 filter-low-complexity benchmark dossier into docs/
	@python3 makes/bin/render_fastq_filter_low_complexity_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_filter_low_complexity_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.filter_low_complexity/corpus-01

_benchmark-filter-reads-corpus-01-report: ## Render the corpus-01 filter-reads benchmark dossier into docs/
	@python3 makes/bin/render_fastq_filter_reads_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_filter_reads_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.filter_reads/corpus-01

_benchmark-remove-duplicates-corpus-01-report: ## Render the corpus-01 remove-duplicates benchmark dossier into docs/
	@python3 makes/bin/render_fastq_remove_duplicates_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_remove_duplicates_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.remove_duplicates/corpus-01

_benchmark-merge-corpus-01-report: ## Render the corpus-01 merge benchmark dossier into docs/
	@python3 makes/bin/render_fastq_merge_pairs_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_merge_pairs_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.merge_pairs/corpus-01

_benchmark-report-qc-corpus-01-report: ## Render the corpus-01 report-qc benchmark dossier into docs/
	@python3 makes/bin/render_fastq_report_qc_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_report_qc_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.report_qc/corpus-01

_benchmark-normalize-primers-corpus-01-report: ## Render the corpus-01 normalize-primers benchmark dossier into docs/
	@python3 makes/bin/render_fastq_normalize_primers_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_normalize_primers_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.normalize_primers/corpus-01

_benchmark-deplete-rrna-corpus-01-report: ## Render the corpus-01 deplete-rrna benchmark dossier into docs/
	@python3 makes/bin/render_fastq_deplete_rrna_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_deplete_rrna_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.deplete_rrna/corpus-01

_benchmark-deplete-host-corpus-01-report: ## Render the corpus-01 deplete-host benchmark dossier into docs/
	@python3 makes/bin/render_fastq_deplete_host_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_deplete_host_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.deplete_host/corpus-01

_benchmark-deplete-reference-contaminants-corpus-01-report: ## Render the corpus-01 deplete-reference-contaminants benchmark dossier into docs/
	@python3 makes/bin/render_fastq_deplete_reference_contaminants_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_deplete_reference_contaminants_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.deplete_reference_contaminants/corpus-01

_benchmark-screen-taxonomy-corpus-01-report: ## Render the corpus-01 screen-taxonomy benchmark dossier into docs/
	@python3 makes/bin/render_fastq_screen_taxonomy_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_screen_taxonomy_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.screen_taxonomy/corpus-01

_benchmark-correct-errors-corpus-01-report: ## Render the corpus-01 correct-errors benchmark dossier into docs/
	@python3 makes/bin/render_fastq_correct_errors_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_correct_errors_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.correct_errors/corpus-01

_benchmark-extract-umis-corpus-01-report: ## Render the corpus-01 extract-umis benchmark dossier into docs/
	@python3 makes/bin/render_fastq_extract_umis_corpus_01_report.py \
		--repo-root . \
		--corpus-root "$(CORPUS_ROOT)" \
		$(if $(OUT_DIR),--run-root "$(OUT_DIR)",)
	@python3 makes/bin/render_fastq_extract_umis_corpus_01_briefing.py \
		--docs-root docs/benchmark/fastq.extract_umis/corpus-01

_benchmark-corpus-01-publication-status: ## Audit corpus-01 FASTQ benchmark publication coverage
	@python3 makes/bin/audit_corpus_01_fastq_benchmark_docs.py \
		--repo-root . \
		--docs-root docs/benchmark \
		--json-out docs/benchmark/corpus-01-status.json \
		--markdown-out docs/benchmark/corpus-01-status.md

_benchmark-corpus-01-published-dossiers: ## Render all published corpus-01 FASTQ dossiers and refresh publication status
	@$(MAKE) _benchmark-validate-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-detect-adapters-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-profile-reads-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-profile-read-lengths-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-profile-overrepresented-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-normalize-primers-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-filter-reads-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-filter-low-complexity-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-deplete-rrna-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-trim-polyg-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-trim-reads-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-trim-terminal-damage-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-merge-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-remove-duplicates-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-report-qc-corpus-01-report CORPUS_ROOT="$(CORPUS_ROOT)" OUT_DIR="$(OUT_DIR)"
	@$(MAKE) _benchmark-corpus-01-publication-status

.PHONY: _benchmark-fastq-stage _benchmark-all _benchmark-trim _benchmark-validate _benchmark-filter \
	_benchmark-merge _benchmark-correct _benchmark-qc-post _benchmark-umi \
	_benchmark-stats _benchmark-screen _benchmark-preprocess _benchmark-status \
	_benchmark-validate-corpus-01 _benchmark-trim-polyg-corpus-01 \
	_benchmark-trim-reads-corpus-01 _benchmark-trim-terminal-damage-corpus-01 \
	_benchmark-detect-adapters-corpus-01 _benchmark-profile-reads-corpus-01 \
	_benchmark-profile-read-lengths-corpus-01 _benchmark-profile-overrepresented-corpus-01 \
	_benchmark-filter-low-complexity-corpus-01 _benchmark-filter-reads-corpus-01 \
	_benchmark-deplete-rrna-corpus-01 _benchmark-screen-taxonomy-corpus-01 \
	_benchmark-correct-errors-corpus-01 _benchmark-extract-umis-corpus-01 \
	_benchmark-merge-corpus-01 _benchmark-report-qc-corpus-01 \
	_benchmark-validate-corpus-01-report _benchmark-trim-polyg-corpus-01-report \
	_benchmark-trim-reads-corpus-01-report _benchmark-trim-terminal-damage-corpus-01-report \
	_benchmark-detect-adapters-corpus-01-report _benchmark-profile-reads-corpus-01-report \
	_benchmark-profile-read-lengths-corpus-01-report \
	_benchmark-profile-overrepresented-corpus-01-report \
	_benchmark-filter-low-complexity-corpus-01-report \
	_benchmark-filter-reads-corpus-01-report _benchmark-deplete-rrna-corpus-01-report _benchmark-screen-taxonomy-corpus-01-report \
	_benchmark-correct-errors-corpus-01-report _benchmark-extract-umis-corpus-01-report _benchmark-merge-corpus-01-report \
	_benchmark-report-qc-corpus-01-report _benchmark-corpus-01-publication-status \
	_benchmark-corpus-01-published-dossiers
