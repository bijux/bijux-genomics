# FASTQ Default Settings (Pre-HPC)

Purpose: define deterministic defaults for every FASTQ stage contract.

## Profile Semantics
- `fastq-to-fastq__default__v1` is the generic FASTQ baseline. It does not require `fastq.trim_terminal_damage` and does not carry terminal-damage tool or parameter defaults.
- `fastq-to-fastq__minimal__v1` is the reduced generic baseline. It keeps only validation, adapter detection, trimming, filtering, and QC aggregation as required stages.
- aDNA-oriented FASTQ profiles opt in to `fastq.trim_terminal_damage` explicitly because terminal damage trimming is a library-specific decision, not an unknown-assay default.
- UMI extraction is ordered immediately after read validation when requested so inline UMIs are captured before adapter, polyG, quality, or length trimming can alter the barcode-bearing sequence.

## Inputs
- FASTQ read pairs or single-end reads, plus optional reference/decoy indexes by stage.

## Outputs
- stage-specific FASTQ/BAM/JSON artifacts declared in stage contracts.

## Key Parameters
- read layout (SE/PE), quality thresholds, adapter/polyg settings, classifier presets.

## Validity Limits
- only pinned tool versions are valid
- contract-required inputs/outputs must be preserved
- stage/tool combinations must remain in index compatibility map

## Stage Coverage
- `fastq.build_contaminant_db`: default `bijux_dna`. rationale: planned internal contract harness records contaminant database build inputs, outputs, and asset-lock expectations until a production backend is admitted.
- `fastq.build_rrna_db`: default `bijux_dna`. rationale: planned internal contract harness records rRNA database build inputs, outputs, and asset-lock expectations until a production backend is admitted.
- `fastq.build_taxonomy_db`: default `bijux_dna`. rationale: planned internal contract harness records taxonomy database build inputs, outputs, and asset-lock expectations until production classifier database builders are admitted.
- `fastq.capture_provenance_snapshot`: default `bijux_dna`. rationale: planned internal contract harness records pre-execution provenance snapshot expectations for governed route and benchmark plans.
- `fastq.classify_layout`: default `bijux_dna`. rationale: planned internal contract harness records read-layout classification inputs and report outputs before production layout classifiers are admitted.
- `fastq.concatenate_lanes`: default `bijux_dna`. rationale: planned internal contract harness records lane-manifest consolidation semantics and output lineage expectations.
- `fastq.deinterleave_reads`: default `bijux_dna`. rationale: planned internal contract harness records interleaved input splitting semantics and paired-output expectations.
- `fastq.demultiplex_reads`: default `bijux_dna`. rationale: planned internal contract harness records barcode-manifest inputs and per-sample output/report expectations.
- `fastq.detect_duplicates_premerge`: default `bijux_dna`. rationale: planned internal contract harness records report-only duplicate-signal expectations without treating the result as scientific duplicate removal.
- `fastq.detect_instrument_artifacts`: default `bijux_dna`. rationale: planned internal contract harness records report-only instrument-artifact expectations before backend-specific detector admission.
- `fastq.estimate_library_complexity_prealign`: default `bijux_dna`. rationale: planned internal contract harness records prealignment library-complexity report expectations without substituting for alignment-derived metrics.
- `fastq.index_reference`: default `bowtie2_build`. rationale: default FASTQ reference preparation must emit the mapper index consumed by governed host and reference-contaminant depletion.
- `fastq.interleave_reads`: default `bijux_dna`. rationale: planned internal contract harness records mate synchronization inputs and interleaved-output expectations.
- `fastq.materialize_qc_manifest`: default `bijux_dna`. rationale: planned internal contract harness records QC report aggregation manifest expectations before production manifest materialization is admitted.
- `fastq.normalize_read_names`: default `bijux_dna`. rationale: planned internal contract harness records read-name rewrite outputs and sidecar mapping expectations.
- `fastq.prepare_adapter_bank`: default `bijux_dna`. rationale: planned internal contract harness records adapter-source hydration and asset-lock expectations for governed trimming plans.
- `fastq.prepare_host_reference_bundle`: default `bijux_dna`. rationale: planned internal contract harness records host-reference source hydration and asset-lock expectations for depletion plans.
- `fastq.prepare_primer_bank`: default `bijux_dna`. rationale: planned internal contract harness records primer-source hydration and asset-lock expectations for amplicon primer-normalization plans.
- `fastq.repair_pairs`: default `bijux_dna`. rationale: planned internal contract harness records mate reconciliation inputs, repaired outputs, and repair-report expectations.
- `fastq.subsample_reads`: default `bijux_dna`. rationale: planned internal contract harness records deterministic subsampling inputs and report expectations for benchmark-comparable plans.
- `fastq.verify_assets`: default `bijux_dna`. rationale: planned internal contract harness records asset-lock verification report expectations before backend-native asset inspectors are admitted.
- `fastq.validate_reads`: default `fastqvalidator`.
- `fastq.profile_read_lengths`: default `seqkit_stats`.
- `fastq.detect_adapters`: default `fastqc`.
- `fastq.trim_polyg_tails`: default `fastp`.
- `fastq.trim_reads`: default `fastp`.
- `fastq.filter_reads`: default `fastp`.
- `fastq.profile_reads`: default `seqkit_stats`.
- `fastq.deplete_rrna`: default `sortmerna`.
- `fastq.report_qc`: default `multiqc`.
- `fastq.merge_pairs`: default `pear`.
- `fastq.remove_duplicates`: default `clumpify`. rationale: the governed default must remain runnable for both single-end and paired-end inputs while still supporting explicit optical-aware duplicate policy.
- `fastq.filter_low_complexity`: default `bbduk`.
- `fastq.deplete_host`: default `bowtie2`.
- `fastq.deplete_reference_contaminants`: default `bowtie2`. rationale: reference-guided decoy depletion stays aligned with the current stage contract.
- `fastq.correct_errors`: default `rcorrector`.
- `fastq.extract_umis`: default `umi_tools`.
- `fastq.profile_overrepresented_sequences`: default `fastqc`.
- `fastq.screen_taxonomy`: default `kraken2`.
- `fastq.trim_terminal_damage`: default `cutadapt`. rationale: deterministic terminal mask/trim policy for aDNA damage-aware pretrim.
- `fastq.normalize_primers`: default `cutadapt`. rationale: deterministic primer trimming with explicit mismatch/orientation controls.
- `fastq.remove_chimeras`: default `vsearch`. rationale: deterministic uchime-based baseline before broader ensemble adoption.
- `fastq.cluster_otus`: default `vsearch`. rationale: stable OTU cluster policy with reproducible identifiers.
- `fastq.infer_asvs`: default `dada2`. rationale: governed ASV inference now uses the admitted containerized dada2 backend and publishes the canonical infer-asvs report plus taxonomy-ready representative sequences.
- `fastq.normalize_abundance`: default `seqkit`. rationale: abundance-table normalization stays within the currently admitted amplicon table tooling.

validation_benchmark_policy: fastq.validate_reads
- default benchmark backend is `fastqvalidator`
- `fastqc`, `fastq_scan`, `fqtools`, and `seqtk` are comparison backends for governed-report agreement studies
- `fastq.report_qc` must consume any governed `fastq.validate_reads` backend artifacts already present in the benchmark output tree, while falling back to bootstrapping `fastqvalidator` when no validation backend has produced report inputs yet
- `fastq.report_qc` and `fastq.profile_reads` are downstream complements, not substitutes for structural validation

profile_read_lengths_benchmark_policy: fastq.profile_read_lengths
- default benchmark backend is `seqkit_stats`
- `fastp`, `prinseq`, and `seqfu` are governed comparison backends for read-length agreement studies
- every governed `fastq.profile_read_lengths` row must emit `read_count`, `min_read_length`, `mean_read_length`, `median_read_length`, and `max_read_length`
- `fastp` must run in report-only mode with trimming and filtering disabled so the benchmark remains pre-trim length profiling instead of quietly reusing a trim contract
- `prinseq` length profiling may stream all output classes to `/dev/null`, but the invocation must remain non-destructive and produce the governed report plus histogram artifacts

detect_adapters_benchmark_policy: fastq.detect_adapters
- default benchmark backend is `fastqc`
- every governed `fastq.detect_adapters` row must emit `adapter_report`, `detected_adapter_ids`, `detection_confidence`, and `detection_threshold`
- governed adapter identities must resolve to adapter-bank IDs instead of raw sequence fragments, and nested partial rescue motifs must not be double-counted when a stronger parent adapter matches the same read
- `fastq.report_qc` may reuse the governed adapter report and evidence directory, but it must not invent new adapter identities beyond the canonical detect-adapters report

trim_reads_benchmark_policy: fastq.trim_reads
- default benchmark backend is `fastp`
- governed comparison backends are `adapterremoval`, `alientrimmer`, `atropos`, `bbduk`, `cutadapt`, `fastx_clipper`, `leehom`, `prinseq`, `seqkit`, `skewer`, `trim_galore`, and `trimmomatic`
- every governed `fastq.trim_reads` row must emit trimmed FASTQ outputs, the governed report output, retained-read count, dropped-read count, and bases-removed accounting
- the retained planned extra `seqpurge` must remain explicitly visible as a non-normalized trim-reads contract until it is admitted and registered instead of silently disappearing from readiness reporting

profile_reads_benchmark_policy: fastq.profile_reads
- default benchmark backend is `seqkit_stats`
- `seqkit` and `seqfu` are governed comparison backends for general read profiling studies
- `fastq.report_qc` must reuse any governed `fastq.profile_reads` backend artifacts already present in the benchmark output tree instead of forcing a second profiling backend run
- the current `seqfu` benchmark route runs through the admitted seqfu compatibility wrapper runtime surface, so its profile-read command line must remain compatible with the wrapped stats entrypoint

single_tool_justification: fastq.index_reference
single_tool_justification: fastq.detect_adapters
single_tool_justification: fastq.deplete_rrna
single_tool_justification: fastq.extract_umis
single_tool_justification: fastq.normalize_primers
single_tool_justification: fastq.remove_chimeras
single_tool_justification: fastq.cluster_otus
single_tool_justification: fastq.normalize_abundance

single_tool_justification: fastq.trim_terminal_damage
