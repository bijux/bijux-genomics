SHELL 			:= /bin/sh
PLATFORM 		?= docker-mac-arm64
JOBS 			?= 8
NEXTEST_JOBS 	?= $(JOBS)
TOOLS_TRIM 		?= fastp,cutadapt,bbduk,adapterremoval,trimmomatic,trim_galore,atropos
TOOLS_VALIDATE 	?= seqtk,fastqc,fastqvalidator,fastqvalidator_official,fqtools
TOOLS_FILTER 	?= prinseq,fastp,seqkit
TOOLS_MERGE 	?= pear,vsearch,bbmerge,flash2
TOOLS_CORRECT 	?= rcorrector
TOOLS_QC_POST 	?= fastqc,multiqc
TOOLS_UMI 		?= umi_tools
TOOLS_STATS 	?= seqkit_stats
TOOLS_SCREEN 	?= kraken2,centrifuge,metaphlan,kaiju,fastq_screen

EXTRA_GOALS := $(filter-out bench-all benchmark-validate benchmark-trim benchmark-merge benchmark-correct benchmark-filter benchmark-stats benchmark-qc-post benchmark-umi benchmark-screen benchmark-preprocess image-qa build-images test-images test-images-trim test-images-validate test-images-filter test-images-merge lint security test coverage test-fast test-slow test-e2e guardrails ci-mac,$(MAKECMDGOALS))
EXTRA_FASTQ_ROOTS := $(EXTRA_GOALS)
FASTQ_ROOT_OVERRIDE ?= $(EXTRA_FASTQ_ROOTS)

.PHONY: build-images test-images image-qa bench-all benchmark-trim benchmark-validate benchmark-filter benchmark-merge \
	benchmark-correct benchmark-qc-post benchmark-umi benchmark-stats benchmark-screen benchmark-preprocess \
	test-images-trim test-images-validate test-images-filter test-images-merge lint quality security test \
	test-fast test-slow test-e2e guardrails ci-mac ci-mac-fast ci-mac-full lint-fast test-full

CARGO_MAKE ?= cargo make
CARGO_MAKE_ENV := JOBS=$(JOBS) NEXTEST_JOBS=$(NEXTEST_JOBS)

test:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) test

test-full:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) test-full

test-fast:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) test-fast

test-slow:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) test-slow

test-e2e:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) test-e2e

guardrails:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) guardrails

coverage:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) coverage

lint:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) lint

security:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) audit

lint-fast:
	@$(CARGO_MAKE_ENV) $(CARGO_MAKE) lint-fast

ci-mac-fast:
	@set -e; \
	if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER=sccache; fi; \
	$(CARGO_MAKE_ENV) $(CARGO_MAKE) lint-fast; \
	$(CARGO_MAKE_ENV) $(CARGO_MAKE) test;

ci-mac-full:
	@set -e; \
	if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER=sccache; fi; \
	$(CARGO_MAKE_ENV) $(CARGO_MAKE) lint; \
	$(CARGO_MAKE_ENV) $(CARGO_MAKE) audit; \
	$(CARGO_MAKE_ENV) $(CARGO_MAKE) test-full; \
	$(CARGO_MAKE_ENV) $(CARGO_MAKE) coverage;

ci-mac: ci-mac-fast
include makefiles/containers.mk
include makefiles/benchmarks.mk
