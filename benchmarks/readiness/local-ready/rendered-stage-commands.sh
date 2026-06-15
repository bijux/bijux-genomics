#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

# fastq.index_reference
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.index_reference
# fastq.validate_reads
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.validate_reads
# fastq.profile_read_lengths
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.profile_read_lengths
# fastq.detect_adapters
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.detect_adapters
# fastq.detect_duplicates_premerge
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.detect_duplicates_premerge
# fastq.estimate_library_complexity_prealign
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.estimate_library_complexity_prealign
# fastq.trim_terminal_damage
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.trim_terminal_damage
# fastq.normalize_primers
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.normalize_primers
# fastq.trim_polyg_tails
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.trim_polyg_tails
# fastq.trim_reads
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.trim_reads
# fastq.filter_reads
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.filter_reads
# fastq.profile_reads
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.profile_reads
# fastq.deplete_rrna
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.deplete_rrna
# fastq.merge_pairs
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.merge_pairs
# fastq.remove_duplicates
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.remove_duplicates
# fastq.filter_low_complexity
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.filter_low_complexity
# fastq.deplete_host
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.deplete_host
# fastq.deplete_reference_contaminants
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.deplete_reference_contaminants
# fastq.correct_errors
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.correct_errors
# fastq.extract_umis
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.extract_umis
# fastq.profile_overrepresented_sequences
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.profile_overrepresented_sequences
# fastq.report_qc
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.report_qc
# fastq.remove_chimeras
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.remove_chimeras
# fastq.infer_asvs
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.infer_asvs
# fastq.cluster_otus
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.cluster_otus
# fastq.normalize_abundance
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.normalize_abundance
# fastq.screen_taxonomy
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id fastq.screen_taxonomy
# bam.align
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.align
# bam.authenticity
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.authenticity
# bam.bias_mitigation
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.bias_mitigation
# bam.complexity
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.complexity
# bam.contamination
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.contamination
# bam.coverage
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.coverage
# bam.damage
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.damage
# bam.duplication_metrics
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.duplication_metrics
# bam.endogenous_content
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.endogenous_content
# bam.filter
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.filter
# bam.gc_bias
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.gc_bias
# bam.genotyping
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.genotyping
# bam.haplogroups
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.haplogroups
# bam.insert_size
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.insert_size
# bam.kinship
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.kinship
# bam.length_filter
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.length_filter
# bam.mapping_summary
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.mapping_summary
# bam.mapq_filter
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.mapq_filter
# bam.markdup
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.markdup
# bam.overlap_correction
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.overlap_correction
# bam.qc_pre
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.qc_pre
# bam.recalibration
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.recalibration
# bam.sex
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.sex
# bam.validate
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.validate
