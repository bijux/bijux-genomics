# BAM Default Settings (Pre-HPC)

Purpose: define deterministic defaults for every BAM stage contract.

## Inputs
- aligned or partially processed BAM, reference assets, optional metadata inputs by stage.

## Outputs
- BAM transforms plus stage metrics/reports defined in stage contracts.

## Key Parameters
- MAPQ/length thresholds, duplicate policy, contamination/authenticity model toggles.

## Validity Limits
- only pinned tool versions are valid
- required stage inputs/outputs must be preserved
- compatibility map in index.yaml is authoritative

## Stage Coverage
- `bam.align`: default `bwa`.
- `bam.validate`: default `samtools`.
- `bam.qc_pre`: default `samtools`.
- `bam.mapping_summary`: default `samtools`.
- `bam.filter`: default `samtools`.
- `bam.mapq_filter`: default `samtools`.
- `bam.length_filter`: default `samtools`.
- `bam.markdup`: default `samtools`.
- `bam.duplication_metrics`: default `samtools`.
- `bam.complexity`: default `preseq`.
- `bam.coverage`: default `mosdepth`.
- `bam.insert_size`: default `picard`.
- `bam.gc_bias`: default `picard`.
- `bam.endogenous_content`: default `samtools`.
- `bam.overlap_correction`: default `bamutil`.
- `bam.damage`: default `mapdamage2`.
- `bam.authenticity`: default `authenticct`.
- `bam.contamination`: default `schmutzi`.
- `bam.sex`: default `rxy`.
- `bam.bias_mitigation`: default `samtools`.
- `bam.recalibration`: default `gatk`.
- `bam.haplogroups`: default `yleaf`.
- `bam.genotyping`: default `gatk`.
- `bam.kinship`: default `king`.

single_tool_justification: bam.complexity
single_tool_justification: bam.insert_size
single_tool_justification: bam.gc_bias
single_tool_justification: bam.endogenous_content
single_tool_justification: bam.overlap_correction
single_tool_justification: bam.bias_mitigation
single_tool_justification: bam.recalibration
single_tool_justification: bam.haplogroups

## Default Rationale
- `bam.align` rationale: prioritize stable alignment baseline with broad BAM ecosystem compatibility.
- `bam.validate` rationale: use deterministic validation diagnostics for contract enforcement.
- `bam.qc_pre` rationale: enforce early sanity checks before downstream filtering while preserving a governed reporting companion for operator-facing aggregation.
- `bam.mapping_summary` rationale: preserve samtools as the governed baseline while keeping a supported Picard comparison row for alignment-summary contract coverage.
- `bam.filter` rationale: minimize post-alignment variance while preserving interpretability.
- `bam.mapq_filter` rationale: deterministic MAPQ gating for reproducible retention metrics.
- `bam.length_filter` rationale: preserve explicit read-length contract boundaries.
- `bam.markdup` rationale: deterministic duplicate marking semantics for repeatable metrics.
- `bam.duplication_metrics` rationale: stable duplicate summaries for comparability.
- `bam.complexity` rationale: planned complexity extrapolation with stable baseline tooling.
- `bam.coverage` rationale: consistent low-overhead depth metrics.
- `bam.insert_size` rationale: deterministic insert-size summaries for QC comparability.
- `bam.gc_bias` rationale: deterministic GC-bias baseline until expanded tool admission.
- `bam.endogenous_content` rationale: reproducible endogenous ratio derivation from mapping summaries.
- `bam.overlap_correction` rationale: deterministic overlap clipping preserves downstream comparability.
- `bam.damage` rationale: preserve historical aDNA comparability baseline.
- `bam.authenticity` rationale: stable authenticity score baseline for operator interpretation.
- `bam.contamination` rationale: established contamination baseline for aDNA workflows.
- `bam.sex` rationale: deterministic ratio-based sex inference baseline.
- `bam.bias_mitigation` rationale: planned baseline keeps deterministic policy until dedicated tools are promoted.
- `bam.recalibration` rationale: planned recalibration baseline remains pinned until full validation.
- `bam.haplogroups` rationale: planned deterministic haplogroup assignment baseline.
- `bam.genotyping` rationale: planned pinned-caller baseline for consistent genotype outputs.
- `bam.kinship` rationale: reproducible pairwise kinship baseline.

## Benchmark Contract Notes
- `bam.align`: the admitted `bwa` and `bowtie2` benchmark rows must declare governed FASTQ input, reference index input, `align.bam`, `align.bam.bai`, and `align.metrics.json`.
- `bam.align`: current readiness stays `artifact_contract_only` until alignment mapping summaries are promoted from artifact presence to normalized BAM parser semantics.
- `bam.validate`: the admitted `samtools`, `bedtools`, and `bamtools` benchmark rows must emit `validation_status`, `validation_errors`, `validation_warnings`, and `input_bam_identity`.
- `bam.validate`: warning-grade findings are currently empty for governed fixtures; validation failures surface through deterministic refusal-code errors instead of a mixed warning/error model.
- `bam.qc_pre`: the admitted `samtools` benchmark row and the governed `multiqc` reporting companion must preserve `total_reads`, `mapped_reads`, `unmapped_reads`, `duplicate_flagged_reads`, and `contig_summary`.
- `bam.qc_pre`: `samtools` remains the primary executor for raw flagstat/idxstats/stats artifacts, while `multiqc` is currently plannable reporting coverage rather than a local-smoke execution backend.
- `bam.mapping_summary`: the admitted `samtools` row and the governed `picard` comparison row must preserve `mapping_fraction`, `mapped_reads`, `unmapped_reads`, `secondary_reads`, and `supplementary_reads`.
- `bam.mapping_summary`: `samtools` remains the fixture-backed execution baseline, while `picard` currently contributes supported comparison coverage through alignment-summary metrics plus governed companion artifacts.
- `bam.filter`: the admitted `samtools`, `bamtools`, and `bedtools` benchmark rows must preserve `input_reads`, `kept_reads`, `removed_reads`, and `active_filters`.
- `bam.filter`: `samtools` remains the local-smoke execution baseline, while `bamtools` and `bedtools` currently contribute supported comparison coverage through the same retained/removed audit-artifact contract.
- `bam.mapq_filter`: the admitted `samtools` and `bamtools` benchmark rows must preserve `mapq_threshold`, `kept_reads`, `removed_reads`, and `filtered_bam`.
- `bam.mapq_filter`: `samtools` remains the fixture-backed MAPQ-gating baseline, while `bamtools` currently contributes supported comparison coverage through the same governed audit-artifact contract and local planning path.
- `bam.length_filter`: the admitted `samtools` and `picard` benchmark rows must preserve `min_length_threshold`, `kept_reads`, `removed_reads`, and `filtered_bam`.
- `bam.length_filter`: `samtools` remains the fixture-backed length-gating baseline, while `picard` currently contributes supported comparison coverage through the same governed audit-artifact contract and local planning path.
- `bam.markdup`: the admitted `samtools` and `picard` benchmark rows must preserve `marked_bam`, `duplicate_metrics`, `duplicate_count`, and `duplicate_fraction`.
- `bam.markdup`: `samtools` remains the fixture-backed duplicate-marking baseline, while `picard` currently contributes supported comparison coverage through the same governed audit-artifact contract and the current GATK MarkDuplicatesSpark planning path.
