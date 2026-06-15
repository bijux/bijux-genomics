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
- `bam.bias_mitigation`: default `mapdamage2`.
- `bam.recalibration`: default `gatk`.
- `bam.haplogroups`: default `yleaf`.
- `bam.genotyping`: default `angsd`.
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
- `bam.complexity` rationale: stable complexity extrapolation baseline with a governed `preseq` execution surface.
- `bam.coverage` rationale: preserve a governed depth-and-breadth comparison surface while keeping `mosdepth` as the low-overhead default.
- `bam.insert_size` rationale: deterministic insert-size summaries for QC comparability.
- `bam.gc_bias` rationale: deterministic GC-bias baseline until expanded tool admission.
- `bam.endogenous_content` rationale: reproducible endogenous ratio derivation from mapping summaries.
- `bam.overlap_correction` rationale: deterministic overlap clipping preserves downstream comparability.
- `bam.damage` rationale: preserve historical aDNA comparability while exposing the full governed damage-tool comparison surface.
- `bam.authenticity` rationale: stable authenticity score baseline for operator interpretation.
- `bam.contamination` rationale: established contamination baseline for aDNA workflows.
- `bam.sex` rationale: deterministic ratio-based sex inference baseline.
- `bam.bias_mitigation` rationale: preserve a governed bias-projection baseline through the admitted `mapdamage2` execution surface.
- `bam.recalibration` rationale: preserve a governed BQSR baseline with owned known-sites and low-coverage skip semantics.
- `bam.haplogroups` rationale: deterministic Y-haplogroup assignment baseline with an owned corpus-backed panel contract.
- `bam.genotyping` rationale: preserve a governed low-depth genotyping baseline with owned candidate-sites and target-regions contracts.
- `bam.kinship` rationale: reproducible pairwise kinship baseline.

## Benchmark Contract Notes
- `bam.align`: the admitted `bwa` and `bowtie2` benchmark rows must declare governed FASTQ input, reference index input, `align.bam`, `align.bam.bai`, and `align.metrics.json`.
- `bam.align`: current readiness stays `artifact_contract_only` until alignment mapping summaries are promoted from artifact presence to normalized BAM parser semantics.
- `bam.validate`: the admitted `samtools`, `bedtools`, and `bamtools` benchmark rows must emit `validation_status`, `validation_errors`, `validation_warnings`, and `input_bam_identity`.
- `bam.validate`: governed local-smoke coverage now uses an explicit tiny binary BAM pass fixture plus a malformed BAM refusal fixture, so deterministic validation pass/refusal behavior is proven through real BAM parsing rather than SAM-text proxy handling.
- `bam.validate`: warning-grade findings are currently empty for governed fixtures; validation failures surface through deterministic refusal-code errors instead of a mixed warning/error model.
- `bam.qc_pre`: the admitted `samtools` benchmark row and the governed `multiqc` reporting companion must preserve `total_reads`, `mapped_reads`, `unmapped_reads`, `duplicate_flagged_reads`, and `contig_summary`.
- `bam.qc_pre`: `samtools` remains the primary executor for raw flagstat/idxstats/stats artifacts, while `multiqc` is currently plannable reporting coverage rather than a local-smoke execution backend.
- `bam.qc_pre`: governed local-smoke coverage now uses the `corpus-01-bam-mini` duplicate-flagged multi-contig alignment fixture so the canonical count and contig metrics are proven through owned corpus inputs rather than toy asset paths.
- `bam.recalibration`: the admitted `gatk` benchmark row must preserve the recalibrated BAM, index, recalibration report, summary JSON, and stage-metrics contract.
- `bam.recalibration`: governed local-smoke coverage now uses the `corpus-01-bam-mini` low-coverage recalibration fixture plus owned known-sites and shared-reference inputs, so coverage-gated skip behavior is proven through owned corpus assets rather than toy paths.
- `bam.mapping_summary`: the admitted `samtools` row and the governed `picard` comparison row must preserve `mapping_fraction`, `mapped_reads`, `unmapped_reads`, `secondary_reads`, and `supplementary_reads`.
- `bam.mapping_summary`: `samtools` remains the fixture-backed execution baseline, while `picard` currently contributes supported comparison coverage through alignment-summary metrics plus governed companion artifacts.
- `bam.mapping_summary`: governed local-smoke coverage now uses the `corpus-01-bam-mini` partial-mapping alignment fixture so the canonical mapped/unmapped split is proven through owned corpus inputs rather than toy asset paths.
- `bam.filter`: the admitted `samtools`, `bamtools`, and `bedtools` benchmark rows must preserve `input_reads`, `kept_reads`, `removed_reads`, and `active_filters`.
- `bam.filter`: `samtools` remains the local-smoke execution baseline, while `bamtools` and `bedtools` currently contribute supported comparison coverage through the same retained/removed audit-artifact contract.
- `bam.filter`: governed local-smoke coverage now uses the `corpus-01-bam-mini` mixed-filter alignment fixture so the retained, low-MAPQ, short-fragment, duplicate, and unmapped branches are proven through owned corpus inputs rather than a transitional toy asset path.
- `bam.mapq_filter`: the admitted `samtools` and `bamtools` benchmark rows must preserve `mapq_threshold`, `kept_reads`, `removed_reads`, and `filtered_bam`.
- `bam.mapq_filter`: `samtools` remains the fixture-backed MAPQ-gating baseline, while `bamtools` currently contributes supported comparison coverage through the same governed audit-artifact contract and local planning path.
- `bam.mapq_filter`: governed local-smoke coverage now uses the `corpus-01-bam-mini` MAPQ-threshold ladder fixture so the retained, threshold-edge, and removed branches are proven through owned corpus inputs rather than a transitional toy asset path.
- `bam.length_filter`: the admitted `samtools` and `picard` benchmark rows must preserve `min_length_threshold`, `kept_reads`, `removed_reads`, and `filtered_bam`.
- `bam.length_filter`: `samtools` remains the fixture-backed length-gating baseline, while `picard` currently contributes supported comparison coverage through the same governed audit-artifact contract and local planning path.
- `bam.length_filter`: governed local-smoke coverage now uses the `corpus-01-bam-mini` length-threshold ladder fixture so retained, threshold-edge, removed, and unmapped branches are proven through owned corpus inputs rather than a transitional toy asset path.
- `bam.markdup`: the admitted `samtools` and `picard` benchmark rows must preserve `marked_bam`, `duplicate_metrics`, `duplicate_count`, and `duplicate_fraction`.
- `bam.markdup`: `samtools` remains the fixture-backed duplicate-marking baseline, while `picard` currently contributes supported comparison coverage through the same governed audit-artifact contract and the current GATK MarkDuplicatesSpark planning path.
- `bam.markdup`: governed local-smoke coverage now uses the `corpus-01-bam-mini` duplicate-cluster fixture so duplicate-primary, duplicate-copy, unique-support, and unmapped-support branches are proven through owned corpus inputs rather than a transitional toy asset path.
- `bam.duplication_metrics`: the admitted `samtools` and `picard` benchmark rows must preserve `duplicate_count`, `duplicate_fraction`, `estimated_library_size`, and the governed duplication histogram/report artifacts.
- `bam.duplication_metrics`: `samtools` remains the fixture-backed duplicate-observation baseline, while `picard` currently contributes supported comparison coverage through the same governed duplicate-burden contract and local planning path.
- `bam.duplication_metrics`: governed local-smoke coverage now uses the `corpus-01-bam-mini` duplicate-cluster fixture so duplicate-family, singleton-family, and insufficient-library-size branches are proven through owned corpus inputs rather than a transitional toy asset path.
- `bam.complexity`: the admitted `preseq` benchmark row must preserve `complexity_curve`, `estimated_library_size`, and `saturation_estimate` across `complexity.json`, `complexity.summary.json`, and `stage.metrics.json`.
- `bam.complexity`: governed local-smoke coverage now uses the `corpus-01-bam-mini` complexity-projection fixture so observed-unique-read extrapolation, estimated library size, and saturation are proven through owned corpus inputs rather than a planner-only toy path.
- `bam.coverage`: the admitted `mosdepth`, `samtools`, and `bedtools` benchmark rows must preserve `mean_depth`, `breadth_1x`, `covered_bases`, `observed_region_count`, and `region_ids` alongside governed region-level coverage output.
- `bam.coverage`: `samtools` remains the fixture-backed local-smoke baseline, while `mosdepth` and `bedtools` contribute supported comparison coverage through the same depth sidecar, coverage summary, and benchmark-facing stage metrics contract.
- `bam.coverage`: governed local-smoke coverage now uses the `corpus-01-bam-mini` target-window coverage fixture so interval depth, breadth, and covered-base expectations are proven through owned BAM and BED inputs rather than a transitional toy asset path.
- `bam.insert_size`: the admitted `picard` benchmark row must preserve `mean_insert_size`, `median_insert_size`, `standard_deviation`, `read_pairs`, and the governed insert-size histogram artifact.
- `bam.insert_size`: the current governed insert-size slice remains a single admitted Picard row, so the durable contract focuses on comparable paired-template metrics and histogram provenance rather than inventing unsupported alternative backends.
- `bam.insert_size`: governed local-smoke coverage now uses the `corpus-01-bam-mini` insert-size triplet fixture so paired-fragment count, mean and median insert size, spread, and histogram provenance are proven through owned corpus inputs rather than a transitional toy asset path.
- `bam.kinship`: the admitted `king` benchmark row must preserve `kinship.json`, `kinship.summary.json`, `kinship.segments.tsv`, and `stage.metrics.json` while reporting `observed_max_overlap_snps`, `pair_count`, `status`, and `pairwise_results` through the pairwise kinship surface.
- `bam.kinship`: governed local-smoke coverage now uses the `corpus-01-bam-mini` low-overlap and related-pair fixtures plus an owned relatedness-panel contract so insufficiency and first-degree expectations are proven through owned corpus inputs instead of transitional toy assets.
- `bam.gc_bias`: the admitted `picard` benchmark row must preserve the governed metrics report, plot, `gc_bias_score`, `at_dropout`, and `gc_dropout` across `gc_bias.summary.json` and `stage.metrics.json`.
- `bam.gc_bias`: the current governed GC-bias slice remains a single admitted Picard row, and the local-smoke benchmark row additionally materializes a `gc_bias.tsv` GC-bin table beside the governed report and plot artifacts instead of implying unsupported alternative backends.
- `bam.gc_bias`: governed local-smoke coverage now uses the `corpus-01-bam-mini` GC-window ladder fixture and its owned reference window FASTA, proving the GC-bin table, summary metrics, and plot/report artifacts through corpus-owned inputs instead of toy assets.
- `bam.endogenous_content`: the admitted `samtools` benchmark row must preserve `total_reads`, `mapped_reads`, `endogenous_reads`, and `endogenous_fraction` across `endogenous.content.json`, `endogenous.summary.json`, and `stage.metrics.json`.
- `bam.endogenous_content`: the current governed endogenous-content slice remains a single admitted samtools row, so the durable contract stays explicit about mapped-read-derived endogenous estimates instead of inventing unsupported comparison backends.
- `bam.endogenous_content`: governed local-smoke coverage now uses the `corpus-01-bam-mini` endogenous partial-mapping fixture so total-read, mapped-read, endogenous-read, and endogenous-fraction expectations are proven through owned corpus inputs instead of toy assets.
- `bam.overlap_correction`: the admitted `bamutil` benchmark row must preserve `overlap_corrected_bam`, `corrected_pairs`, `corrected_overlap_bases`, and the governed before-and-after audit artifacts across `overlap_correction.summary.json` and `stage.metrics.json`.
- `bam.overlap_correction`: the current governed overlap-correction slice remains a single admitted `bamutil` row, and the repo no longer advertises `samtools` as a planned overlap-correction benchmark backend because the current governed contract does not own a distinct `samtools` overlap-clipping surface.
- `bam.overlap_correction`: governed local-smoke coverage now uses the `corpus-01-bam-mini` paired-overlap control fixture so corrected-pair counts, corrected-base counts, and before-and-after audit artifacts are proven through owned corpus inputs instead of a transitional toy SAM path.
- `bam.damage`: the governed damage benchmark surface must preserve `damage.summary.json`, `damage.unified_metrics.json`, `damage.parser_output.json`, and `stage.metrics.json` across `mapdamage2`, `pydamage`, `damageprofiler`, `addeam`, `pmdtools`, and `ngsbriggs`.
- `bam.damage`: `mapdamage2`, `pydamage`, `damageprofiler`, `addeam`, `pmdtools`, and `ngsbriggs` stay visible as admitted damage-tool comparison rows when their governed contracts are present.
- `bam.authenticity`: the admitted `authenticct`, `damageprofiler`, and `pmdtools` benchmark rows must preserve `authenticity.json`, `authenticity.summary.json`, and `stage.metrics.json` while reporting `score`, `confidence`, `pmd_like_signal_present`, `consumed_metric_ids`, and `missing_metric_ids` from the governed composed evidence surface.
- `bam.authenticity`: governed local-smoke coverage now uses the `corpus-01-bam-mini` ancient-like damage fixture so authenticity score, confidence, PMD-like signal, and composed evidence availability are proven through owned corpus inputs instead of a transitional toy SAM path.
- `bam.contamination`: the admitted `schmutzi`, `verifybamid2`, and `contammix` benchmark rows must preserve `contamination.json`, `contamination.summary.json`, and `stage.metrics.json` while reporting `scope`, `prerequisites_passed`, `estimate`, `ci_low`, and `ci_high` through the governed panel-backed evidence contract.
- `bam.contamination`: governed local-ready planning now uses the `corpus-01-bam-mini` contamination-panel fixture plus an owned population-AF panel so BAM, reference, and panel inputs are proven through corpus-owned paths instead of transitional toy assets.
- `bam.sex`: the admitted `rxy`, `angsd`, and `yleaf` benchmark rows must preserve `sex.json`, `sex.summary.json`, and `stage.metrics.json` while reporting `x_coverage`, `y_coverage`, `autosomal_coverage`, `call`, `confidence`, and `status` through the governed sex report surface.
- `bam.sex`: governed local-smoke coverage now uses the `corpus-01-bam-mini` XY-autosome coverage fixture and shared BAM reference so sex-inference coverage expectations are proven through owned corpus inputs instead of transitional toy assets.
- `bam.bias_mitigation`: the admitted `mapdamage2` benchmark row must preserve `bias.json`, `bias.summary.json`, and `stage.metrics.json` while reporting `gc_bias_score`, `pre_mitigation_metric`, `post_mitigation_metric`, and mitigation actions through the governed bias-mitigation report surface.
- `bam.bias_mitigation`: governed local-smoke coverage now uses the `corpus-01-bam-mini` GC-window ladder fixture and owned reference FASTA so before-and-after bias projections are proven through corpus-owned inputs instead of transitional toy assets.
