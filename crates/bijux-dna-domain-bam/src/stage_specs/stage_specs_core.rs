use crate::params::{
    AlignEffectiveParams, BamEffectiveParams, ComplexityEffectiveParams, CoverageEffectiveParams,
    DuplicateAction, EndogenousContentEffectiveParams, FilterEffectiveParams,
    MarkDupEffectiveParams, OpticalDuplicatePolicy, QcPreEffectiveParams, ReadGroupSpec, UmiPolicy,
    ValidateEffectiveParams,
};
use crate::{ArtifactPolicy, AuditArtifact, BamStage, BamStageSpec};

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn required_audit_artifacts(stage: BamStage) -> &'static [AuditArtifact] {
    match stage {
        BamStage::Align => &[
            AuditArtifact { name: "align_bam", filename: "align.bam" },
            AuditArtifact { name: "align_bai", filename: "align.bam.bai" },
            AuditArtifact { name: "flagstat", filename: "flagstat.txt" },
            AuditArtifact { name: "idxstats", filename: "idxstats.txt" },
            AuditArtifact { name: "stats", filename: "samtools_stats.txt" },
            AuditArtifact { name: "align_metrics", filename: "align.metrics.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Validate => &[
            AuditArtifact { name: "validation_report", filename: "validation.json" },
            AuditArtifact { name: "flagstat", filename: "flagstat.txt" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::QcPre => &[
            AuditArtifact { name: "flagstat", filename: "flagstat.txt" },
            AuditArtifact { name: "idxstats", filename: "idxstats.txt" },
            AuditArtifact { name: "stats", filename: "samtools_stats.txt" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::MappingSummary => &[
            AuditArtifact { name: "flagstat", filename: "flagstat.txt" },
            AuditArtifact { name: "idxstats", filename: "idxstats.txt" },
            AuditArtifact { name: "stats", filename: "samtools_stats.txt" },
            AuditArtifact { name: "summary", filename: "mapping.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Filter => &[
            AuditArtifact { name: "filtered_bam", filename: "filtered.bam" },
            AuditArtifact { name: "filtered_bai", filename: "filtered.bam.bai" },
            AuditArtifact { name: "flagstat_before", filename: "flagstat.before.txt" },
            AuditArtifact { name: "flagstat_after", filename: "flagstat.after.txt" },
            AuditArtifact { name: "idxstats_before", filename: "idxstats.before.txt" },
            AuditArtifact { name: "idxstats_after", filename: "idxstats.after.txt" },
            AuditArtifact { name: "summary", filename: "filter.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::MapqFilter => &[
            AuditArtifact { name: "filtered_bam", filename: "filtered.bam" },
            AuditArtifact { name: "filtered_bai", filename: "filtered.bam.bai" },
            AuditArtifact { name: "flagstat_before", filename: "flagstat.before.txt" },
            AuditArtifact { name: "flagstat_after", filename: "flagstat.after.txt" },
            AuditArtifact { name: "summary", filename: "mapq_filter.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::LengthFilter => &[
            AuditArtifact { name: "filtered_bam", filename: "filtered.bam" },
            AuditArtifact { name: "filtered_bai", filename: "filtered.bam.bai" },
            AuditArtifact { name: "summary", filename: "length_filter.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Markdup => &[
            AuditArtifact { name: "markdup_bam", filename: "markdup.bam" },
            AuditArtifact { name: "markdup_bai", filename: "markdup.bam.bai" },
            AuditArtifact { name: "flagstat_before", filename: "flagstat.before.txt" },
            AuditArtifact { name: "flagstat_after", filename: "flagstat.after.txt" },
            AuditArtifact { name: "idxstats_before", filename: "idxstats.before.txt" },
            AuditArtifact { name: "idxstats_after", filename: "idxstats.after.txt" },
            AuditArtifact { name: "summary", filename: "markdup.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::DuplicationMetrics => &[
            AuditArtifact { name: "duplication_report", filename: "duplication.metrics.json" },
            AuditArtifact { name: "duplication_histogram", filename: "duplication.histogram.txt" },
            AuditArtifact { name: "summary", filename: "duplication.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Complexity => &[
            AuditArtifact { name: "complexity_report", filename: "complexity.json" },
            AuditArtifact { name: "complexity_curve", filename: "complexity_curve.tsv" },
            AuditArtifact { name: "summary", filename: "complexity.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Coverage => &[
            AuditArtifact { name: "coverage_summary", filename: "coverage.mosdepth.summary.txt" },
            AuditArtifact { name: "coverage_depth", filename: "coverage.depth.txt" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::InsertSize => &[
            AuditArtifact { name: "insert_size_report", filename: "insert_size.metrics.txt" },
            AuditArtifact { name: "insert_size_histogram", filename: "insert_size.histogram.pdf" },
            AuditArtifact { name: "summary", filename: "insert_size.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::GcBias => &[
            AuditArtifact { name: "gc_bias_report", filename: "gc_bias.metrics.txt" },
            AuditArtifact { name: "gc_bias_plot", filename: "gc_bias.plot.pdf" },
            AuditArtifact { name: "summary", filename: "gc_bias.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::EndogenousContent => &[
            AuditArtifact { name: "endogenous_report", filename: "endogenous.content.json" },
            AuditArtifact { name: "summary", filename: "endogenous.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::OverlapCorrection => &[
            AuditArtifact { name: "overlap_corrected_bam", filename: "overlap.corrected.bam" },
            AuditArtifact { name: "overlap_corrected_bai", filename: "overlap.corrected.bam.bai" },
            AuditArtifact { name: "summary", filename: "overlap_correction.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Damage => &[
            AuditArtifact { name: "damage_pydamage", filename: "damage.pydamage.json" },
            AuditArtifact { name: "damage_mapdamage2", filename: "damage.mapdamage2.txt" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Authenticity => &[
            AuditArtifact { name: "authenticity_report", filename: "authenticity.json" },
            AuditArtifact { name: "summary", filename: "authenticity.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Contamination => &[
            AuditArtifact { name: "contamination_report", filename: "contamination.json" },
            AuditArtifact { name: "summary", filename: "contamination.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Sex => &[
            AuditArtifact { name: "sex_report", filename: "sex.json" },
            AuditArtifact { name: "summary", filename: "sex.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::BiasMitigation => &[
            AuditArtifact { name: "bias_report", filename: "bias.json" },
            AuditArtifact { name: "summary", filename: "bias.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Recalibration => &[
            AuditArtifact { name: "recal_bam", filename: "recal.bam" },
            AuditArtifact { name: "recal_bai", filename: "recal.bam.bai" },
            AuditArtifact { name: "recal_report", filename: "recal.report.txt" },
            AuditArtifact { name: "summary", filename: "recal.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Haplogroups => &[
            AuditArtifact { name: "haplogroups", filename: "haplogroups.json" },
            AuditArtifact { name: "summary", filename: "haplogroups.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Genotyping => &[
            AuditArtifact { name: "genotyping_report", filename: "genotyping.json" },
            AuditArtifact { name: "summary", filename: "genotyping.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
        BamStage::Kinship => &[
            AuditArtifact { name: "kinship_report", filename: "kinship.json" },
            AuditArtifact { name: "summary", filename: "kinship.summary.json" },
            AuditArtifact { name: "stage_metrics", filename: "stage.metrics.json" },
        ],
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn stage_spec_core(stage: BamStage) -> Option<BamStageSpec> {
    let spec = match stage {
        BamStage::Align => BamStageSpec {
            stage,
            required_inputs: &["fastq_r1"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "align_bam",
                    "align_bai",
                    "flagstat",
                    "idxstats",
                    "stats",
                    "align_metrics",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::Align(AlignEffectiveParams {
                aligner: "bwa".to_string(),
                strategy_id: "bwa_mem_default".to_string(),
                preset: "default".to_string(),
                mode: "end_to_end".to_string(),
                threads: 1,
                reference: "reference.fasta".to_string(),
                reference_digest: "unknown".to_string(),
                rg_policy: crate::types::ReadGroupPolicy::Regenerate,
                read_group: ReadGroupSpec::with_defaults("sample"),
                sensitivity_profile: Some("default".to_string()),
                seed_length: None,
                build_indices: false,
                emit_stats: true,
            }),
        },
        BamStage::Validate => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["validation_report", "flagstat", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::Validate(ValidateEffectiveParams { strict: true }),
        },
        BamStage::QcPre => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["flagstat", "idxstats", "stats", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::QcPre(QcPreEffectiveParams { regions: None }),
        },
        BamStage::MappingSummary => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["flagstat", "idxstats", "stats", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::MappingSummary(QcPreEffectiveParams {
                regions: None,
            }),
        },
        BamStage::Filter => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "filtered_bam",
                    "filtered_bai",
                    "flagstat_before",
                    "flagstat_after",
                    "idxstats_before",
                    "idxstats_after",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::Filter(FilterEffectiveParams {
                mapq_threshold: 30,
                include_flags: Vec::new(),
                exclude_flags: Vec::new(),
                min_length: 30,
                remove_duplicates: false,
                base_quality_threshold: 20,
            }),
        },
        BamStage::MapqFilter => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "filtered_bam",
                    "filtered_bai",
                    "flagstat_before",
                    "flagstat_after",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::MapqFilter(FilterEffectiveParams {
                mapq_threshold: 30,
                include_flags: Vec::new(),
                exclude_flags: Vec::new(),
                min_length: 0,
                remove_duplicates: false,
                base_quality_threshold: 20,
            }),
        },
        BamStage::LengthFilter => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["filtered_bam", "filtered_bai", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::LengthFilter(FilterEffectiveParams {
                mapq_threshold: 0,
                include_flags: Vec::new(),
                exclude_flags: Vec::new(),
                min_length: 30,
                remove_duplicates: false,
                base_quality_threshold: 20,
            }),
        },
        BamStage::Markdup => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "markdup_bam",
                    "markdup_bai",
                    "flagstat_before",
                    "flagstat_after",
                    "idxstats_before",
                    "idxstats_after",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::Markdup(MarkDupEffectiveParams {
                optical_duplicates: OpticalDuplicatePolicy::MarkOnly,
                umi_policy: UmiPolicy::Ignore,
                duplicate_action: DuplicateAction::Mark,
            }),
        },
        BamStage::DuplicationMetrics => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "duplication_report",
                    "duplication_histogram",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::DuplicationMetrics(MarkDupEffectiveParams {
                optical_duplicates: OpticalDuplicatePolicy::MarkOnly,
                umi_policy: UmiPolicy::Ignore,
                duplicate_action: DuplicateAction::Mark,
            }),
        },
        BamStage::Complexity => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "complexity_report",
                    "complexity_curve",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::Complexity(ComplexityEffectiveParams {
                min_reads: 100_000,
                projection_points: vec![1_000_000, 2_000_000],
            }),
        },
        BamStage::Coverage => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["coverage_summary", "coverage_depth", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::Coverage(CoverageEffectiveParams {
                regions: None,
                depth_thresholds: vec![1, 3, 5],
                regime_mode: "advisory_and_enforced".to_string(),
            }),
        },
        BamStage::InsertSize => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "insert_size_report",
                    "insert_size_histogram",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::InsertSize(CoverageEffectiveParams {
                regions: None,
                depth_thresholds: vec![1],
                regime_mode: "advisory_and_enforced".to_string(),
            }),
        },
        BamStage::GcBias => BamStageSpec {
            stage,
            required_inputs: &["bam", "reference"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["gc_bias_report", "gc_bias_plot", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::GcBias(CoverageEffectiveParams {
                regions: None,
                depth_thresholds: vec![1],
                regime_mode: "advisory_and_enforced".to_string(),
            }),
        },
        BamStage::EndogenousContent => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["endogenous_report", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::EndogenousContent(
                EndogenousContentEffectiveParams {
                    regions: None,
                    depth_thresholds: vec![1],
                    host_reference_scope: "host_reference_required".to_string(),
                    host_reference_digest: None,
                    refuse_without_host_reference: true,
                },
            ),
        },
        BamStage::OverlapCorrection => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "overlap_corrected_bam",
                    "overlap_corrected_bai",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            default_params: BamEffectiveParams::OverlapCorrection(FilterEffectiveParams {
                mapq_threshold: 0,
                include_flags: Vec::new(),
                exclude_flags: Vec::new(),
                min_length: 0,
                remove_duplicates: false,
                base_quality_threshold: 20,
            }),
        },
        _ => return None,
    };
    Some(spec)
}
