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
- `fastq.detect_duplicates_premerge`: default `bijux_dna`. rationale: governed internal contract harness records report-only duplicate-signal expectations from the owned paired-end corpus without treating the result as scientific duplicate removal.
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

screen_taxonomy_benchmark_policy: fastq.screen_taxonomy
- default benchmark backend is `kraken2`
- governed comparison backends are `centrifuge`, `kaiju`, and `krakenuniq`
- retained planned taxonomy row `diamond` must remain visible as non-admitted registry drift until it is either fully registered and normalized or explicitly removed from the benchmark matrix
- every governed `fastq.screen_taxonomy` row must emit `classified_reads`, `unclassified_reads`, `top_taxa`, and `taxonomy_database_id`
- `taxonomy_database_id` must stay aligned with the governed `database_artifact_id`, and the classified versus unclassified read counts must stay derived from the governed read totals plus taxonomy fractions so backend comparisons do not require classifier-specific summary parsing

merge_pairs_benchmark_policy: fastq.merge_pairs
- default benchmark backend is `pear`
- governed comparison backends are `adapterremoval`, `bbmerge`, `flash2`, `leehom`, and `vsearch`
- the current governed benchmark surface is assigned to `fixture:corpus-01-mini`
- every governed `fastq.merge_pairs` row must emit `merged_pair_count`, `unmerged_pair_count`, `discarded_pair_count`, and `merge_rate`
- benchmark rows must keep `input_pair_count`, `merged_pair_count`, and `unmerged_pair_count` aligned with the governed `reads_r1`, `reads_r2`, `reads_merged`, and `reads_unmerged` report fields so downstream comparison code does not need backend-specific merge math
- `discarded_pair_count` must stay derived from the governed pair counts instead of inventing a backend-specific discard metric that some merge tools do not publish directly

extract_umis_benchmark_policy: fastq.extract_umis
- default benchmark backend is `umi_tools`
- every governed `fastq.extract_umis` row must emit `umi_pattern`, `extracted_umi_count`, `invalid_umi_count`, and `tag_header_format`
- benchmark rows must preserve the governed downstream propagation policy and the UMI-tagged FASTQ output paths so header tagging remains distinguishable from downstream BAM-tag materialization expectations
- the current governed benchmark surface is intentionally single-tool; completing the row contract must not be misrepresented as admitted alternative-backend coverage

profile_read_lengths_benchmark_policy: fastq.profile_read_lengths
- default benchmark backend is `seqkit_stats`
- `fastp`, `prinseq`, and `seqfu` are governed comparison backends for read-length agreement studies
- every governed `fastq.profile_read_lengths` row must emit `read_count`, `min_read_length`, `mean_read_length`, `median_read_length`, and `max_read_length`
- `fastp` must run in report-only mode with trimming and filtering disabled so the benchmark remains pre-trim length profiling instead of quietly reusing a trim contract
- `prinseq` length profiling may stream all output classes to `/dev/null`, but the invocation must remain non-destructive and produce the governed report plus histogram artifacts

detect_adapters_benchmark_policy: fastq.detect_adapters
- default benchmark backend is `fastqc`
- the current governed benchmark surface is intentionally single-tool and assigned to `fixture:corpus-01-mini`
- every governed `fastq.detect_adapters` row must emit `adapter_report`, `detected_adapter_ids`, `detection_confidence`, and `detection_threshold`
- governed adapter identities must resolve to adapter-bank IDs instead of raw sequence fragments, and nested partial rescue motifs must not be double-counted when a stronger parent adapter matches the same read
- `fastq.report_qc` may reuse the governed adapter report and evidence directory, but it must not invent new adapter identities beyond the canonical detect-adapters report

detect_duplicates_premerge_benchmark_policy: fastq.detect_duplicates_premerge
- default benchmark backend is the planned internal `bijux_dna` contract harness
- the current governed benchmark surface is assigned to `fixture:corpus-01-mini`
- every governed `fastq.detect_duplicates_premerge` row must emit `duplicate_count`, `duplicate_fraction`, and `inspected_pair_count`
- benchmark rows must preserve the report-only `duplicate_detection_policy` and `measurement_scope` so premerge duplicate signaling is not misrepresented as scientific duplicate removal
- paired-end rows must report inspected pair count from the governed sequence-signature comparison, while single-end rows must leave inspected pair count empty instead of inventing a synthetic pair total

estimate_library_complexity_prealign_benchmark_policy: fastq.estimate_library_complexity_prealign
- default benchmark backend is the planned internal `bijux_dna` contract harness
- the current governed benchmark smoke surface is assigned to `fixture:corpus-01-mini`
- every governed `fastq.estimate_library_complexity_prealign` row must emit `estimated_complexity` or a deterministic `insufficient_data_reason`
- benchmark rows must preserve the governed `complexity_policy` and `estimate_method` so prealignment k-mer heuristics are not confused with alignment-derived library-complexity surfaces
- `estimated_complexity` must stay aligned with `estimated_unique_fraction` when the estimator has enough reads, while insufficient rows must leave `estimated_complexity` empty instead of overloading zero as a successful estimate
- the stage remains advisory-only and declared-only until a normalized runtime benchmark cohort is admitted; completing the row contract or fixture ownership must not be misrepresented as broader execution maturity

trim_polyg_tails_benchmark_policy: fastq.trim_polyg_tails
- default benchmark backend is `fastp`
- governed comparison backend is `bbduk`
- the current governed benchmark surface is assigned to `fixture:corpus-01-mini`
- every governed `fastq.trim_polyg_tails` row must emit `trimmed_tail_count` plus removed-base count through `bases_trimmed_polyg`
- the governed trim-polyg report must preserve backend-native report provenance while publishing canonical tail-count and removed-base metrics for downstream comparison

trim_reads_benchmark_policy: fastq.trim_reads
- default benchmark backend is `fastp`
- governed comparison backends are `adapterremoval`, `alientrimmer`, `atropos`, `bbduk`, `cutadapt`, `fastx_clipper`, `leehom`, `prinseq`, `seqkit`, `skewer`, `trim_galore`, and `trimmomatic`
- every governed `fastq.trim_reads` row must emit trimmed FASTQ outputs, the governed report output, retained-read count, dropped-read count, and bases-removed accounting
- the retained planned extra `seqpurge` must remain explicitly visible as a non-normalized trim-reads contract until it is admitted and registered instead of silently disappearing from readiness reporting

filter_reads_benchmark_policy: fastq.filter_reads
- default benchmark backend is `fastp`
- governed comparison backends are `bbduk`, `prinseq`, and `seqkit`
- the current governed benchmark surface is assigned to `fixture:corpus-01-mini`
- every governed `fastq.filter_reads` row must emit filtered FASTQ outputs, retained-read count, removed-read count, and per-reason removal accounting for `n`, `entropy`, `low_complexity`, `kmer`, `contaminant_kmer`, and `length`
- the explicit `reads_retained` and `reads_removed` aliases must stay aligned with `reads_out` and `reads_dropped` so downstream comparison code does not need to infer benchmark row semantics from stage-internal metric names

correct_errors_benchmark_policy: fastq.correct_errors
- default benchmark backend is `rcorrector`
- governed comparison backends are `bayeshammer`, `lighter`, and `musket`
- every governed `fastq.correct_errors` row must emit `corrected_reads`, `changed_reads`, `unchanged_reads`, and the governed corrected FASTQ output paths
- `changed_reads` and `unchanged_reads` must come from real input-versus-output FASTQ comparison instead of inferring correction activity from aggregate quality or base totals
- the canonical `corrected_reads` count remains the emitted corrected-output read count; it must not be repurposed to mean only reads whose content changed

deplete_rrna_benchmark_policy: fastq.deplete_rrna
- default benchmark backend is `sortmerna`
- the current governed benchmark surface remains single-tool until the domain contract admits additional normalized depletion backends
- every governed `fastq.deplete_rrna` row must emit `rrna_db`, `retained_reads`, `removed_reads`, and `depletion_rate`
- `retained_reads` must stay aligned with `reads_out`, `removed_reads` must stay aligned with `reads_removed`, and `depletion_rate` must stay aligned with `rrna_fraction_removed` so downstream comparison code does not need stage-specific translation
- the benchmark row must preserve the governed retained-read role, removed-read role, and retained FASTQ output path so depletion accounting stays auditable even while the truthful benchmark cohort is still single-tool

deplete_host_benchmark_policy: fastq.deplete_host
- default benchmark backend is `bowtie2`
- the current governed benchmark surface remains single-tool until the benchmark registry admits an alternative host-depletion backend such as `star`
- every governed `fastq.deplete_host` row must emit `host_index_artifact_id`, retained FASTQ outputs, removed-host FASTQ outputs, `depleted_reads`, and `host_hit_rate`
- `host_index_artifact_id` must stay aligned with the governed `reference_index_artifact_id`, `depleted_reads` must stay aligned with `reads_removed`, and `host_hit_rate` must stay aligned with `host_fraction_removed`
- the benchmark row must preserve the governed retained-read policy and removed-host output provenance so host-hit accounting remains auditable without pretending optional domain-level alternatives are already benchmark-admitted

deplete_reference_contaminants_benchmark_policy: fastq.deplete_reference_contaminants
- default benchmark backend is `bowtie2`
- the current governed benchmark surface remains single-tool until the benchmark registry admits an alternative contaminant-depletion backend
- every governed `fastq.deplete_reference_contaminants` row must emit `contaminant_index_artifact_id`, retained FASTQ outputs, `contaminant_reads`, and `contaminant_hit_rate`
- `contaminant_index_artifact_id` must stay aligned with the governed `reference_index_artifact_id`, `contaminant_reads` must stay aligned with `reads_removed`, and `contaminant_hit_rate` must stay aligned with `contaminant_fraction_removed`
- the benchmark row must preserve the governed retained-read role and contaminant reference identity so decoy-screen accounting remains auditable without overstating tool admission

trim_terminal_damage_benchmark_policy: fastq.trim_terminal_damage
- default benchmark backend is `cutadapt`
- governed comparison backends are `adapterremoval` and `seqkit`
- the current governed benchmark surface is assigned to `fixture:corpus-01-mini`
- every governed `fastq.trim_terminal_damage` row must emit `trim_5p_bases`, `trim_3p_bases`, `reads_retained`, and `bases_removed`
- the benchmark row must preserve the governed execution policy and UDG classification so explicit trimming remains distinguishable from preserve-ended policy results

remove_duplicates_benchmark_policy: fastq.remove_duplicates
- default benchmark backend is `clumpify`
- governed comparison backend is `fastuniq`
- the current governed benchmark surface is assigned to `fixture:corpus-01-mini`
- every governed `fastq.remove_duplicates` row must emit `input_reads`, `duplicate_reads`, `unique_reads`, and `output_reads`
- benchmark rows must keep the stage-native `reads_in`, `reads_out`, and `duplicates_removed` fields aligned with those canonical aliases so deduplication comparisons do not need stage-specific name translation
- paired-end rows must preserve the pair-count coherence contract alongside the read-count aliases, and single-end rows must leave pair fields empty instead of inventing synthetic mate counts

filter_low_complexity_benchmark_policy: fastq.filter_low_complexity
- default benchmark backend is `bbduk`
- governed comparison backend is `prinseq`
- the current governed benchmark surface is assigned to `fixture:corpus-01-mini`
- every governed `fastq.filter_low_complexity` row must emit `reads_removed_low_complexity` plus filtered FASTQ output paths
- the canonical benchmark row must publish those output paths as `filtered_fastq_r1` and `filtered_fastq_r2` while preserving the governed low-complexity removal count from the report contract
- retained planned rows for `dustmasker` and `fastp` must remain visible as non-admitted drift until they are either fully admitted and registered or explicitly removed from the benchmark matrix

profile_reads_benchmark_policy: fastq.profile_reads
- default benchmark backend is `seqkit_stats`
- `seqkit` and `seqfu` are governed comparison backends for general read profiling studies
- `fastq.report_qc` must reuse any governed `fastq.profile_reads` backend artifacts already present in the benchmark output tree instead of forcing a second profiling backend run
- the current `seqfu` benchmark route runs through the admitted seqfu compatibility wrapper runtime surface, so its profile-read command line must remain compatible with the wrapped stats entrypoint

index_reference_benchmark_policy: fastq.index_reference
- default benchmark backend is `bowtie2_build`
- `star` remains an admitted comparison backend for reference-index preparation cost studies
- every governed `fastq.index_reference` row must emit `index_directory`, `index_files`, `elapsed_time_s`, and `index_size_bytes`
- `index_directory` must stay aligned with the governed emitted index root, `index_files` must preserve the governed emitted file list with byte counts, `elapsed_time_s` must stay aligned with `runtime_s`, and `index_size_bytes` must stay aligned with `index_bytes`
- the benchmark row must preserve the selected index format so downstream reference-guided stages can compare preparation cost without obscuring mapper-specific index ownership

normalize_primers_benchmark_policy: fastq.normalize_primers
- default benchmark backend is `cutadapt`
- the current governed benchmark surface is intentionally single-tool and assigned to `fixture:corpus-03-amplicon-mini`
- every governed `fastq.normalize_primers` row must emit `matched_primers`, `unmatched_reads`, `trimmed_primer_bases`, and the normalized FASTQ outputs
- `matched_primers` must stay aligned with the governed `primer_trimmed_reads` count, `unmatched_reads` must stay derived from `reads_in - matched_primers`, and `trimmed_primer_bases` must stay derived from `bases_in - bases_out`
- the normalized FASTQ output aliases must keep pointing at the governed `normalized_reads_r1` and `normalized_reads_r2` artifacts so downstream amplicon stages inherit the same primer-normalized read identity

remove_chimeras_benchmark_policy: fastq.remove_chimeras
- default benchmark backend is `vsearch`
- the current governed benchmark surface is intentionally single-tool and assigned to `fixture:corpus-03-amplicon-mini`
- every governed `fastq.remove_chimeras` row must emit `chimera_count`, `non_chimera_count`, and `filtered_representative_sequences`
- `chimera_count` must stay aligned with the governed `chimeras_removed` count, `non_chimera_count` must stay aligned with `reads_out`, and `filtered_representative_sequences` must stay aligned with the governed filtered-output artifact path
- the admitted benchmark row must preserve the governed method and detection-scope identity so the deterministic UCHIME baseline remains auditable while the truthful cohort is still single-tool

infer_asvs_benchmark_policy: fastq.infer_asvs
- default benchmark backend is `dada2`
- the current governed benchmark surface is intentionally single-tool and assigned to `fixture:corpus-03-amplicon-mini`
- every governed `fastq.infer_asvs` row must emit the ASV abundance table path, representative-sequence FASTA path, `asv_count`, and `sample_count`
- the benchmark aliases must stay aligned with the governed report contract: `asv_table_tsv` points at the canonical abundance table and `representative_sequences_fasta` points at the canonical representative-sequence FASTA
- `asv_count` must stay aligned with the inferred feature count in the governed ASV table, and `sample_count` must stay aligned with the distinct sample count represented in that table

cluster_otus_benchmark_policy: fastq.cluster_otus
- default benchmark backend is `vsearch`
- the current governed benchmark surface is intentionally single-tool and assigned to `fixture:corpus-03-amplicon-mini`
- every governed `fastq.cluster_otus` row must emit `otu_table_tsv`, `representative_sequences_fasta`, `otu_count`, and `clustering_threshold`
- the benchmark aliases must stay aligned with the governed report contract: `otu_table_tsv` points at the canonical OTU abundance table, `representative_sequences_fasta` points at the canonical representative-sequence FASTA, and `clustering_threshold` stays aligned with the governed `otu_identity` parameter
- `otu_count` must stay aligned with the distinct OTU identifiers represented in the governed abundance table, and the admitted benchmark row must preserve the configured identity threshold so OTU clustering comparisons stay auditable

normalize_abundance_benchmark_policy: fastq.normalize_abundance
- default benchmark backend is `seqkit`
- the current governed benchmark surface admits `seqkit` as the runnable benchmark row and retains `seqfu` as planned registry drift until it is either fully admitted or explicitly removed
- every governed `fastq.normalize_abundance` row must emit `normalized_abundance_tsv`, `sample_totals`, `normalization_method`, and `numeric_output_valid`
- `normalized_abundance_tsv` must stay aligned with the canonical governed normalized feature table, and `normalization_method` must stay aligned with the governed `method` parameter selected for the run
- `sample_totals` must preserve the governed per-sample compositional sums, and `numeric_output_valid` must stay true only when those sums match the expected scale implied by `scale_factor` or the unit-total normalization rule
- the planned `seqfu` row must remain explicitly visible in readiness and registry-drift reporting until the repo really admits a normalized runtime and registry contract for it

single_tool_justification: fastq.detect_adapters
single_tool_justification: fastq.deplete_rrna
single_tool_justification: fastq.extract_umis
single_tool_justification: fastq.remove_chimeras
single_tool_justification: fastq.cluster_otus
single_tool_justification: fastq.normalize_abundance
