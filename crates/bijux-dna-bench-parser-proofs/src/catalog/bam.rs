use super::{FamilyProbeSpec, ProbeKind};
use crate::reports::ParseKind;

pub(super) const BAM: &[FamilyProbeSpec] = &[
    FamilyProbeSpec {
        family_id: "alignment",
        representative_stage_id: "bam.align",
        probe_kind: ProbeKind::GeneratedBamAlign,
        parse_kind: ParseKind::BamAlignmentProvenance,
        snapshot_keys: &["backend_tool_id", "strategy_id", "preset", "mode"],
    },
    FamilyProbeSpec {
        family_id: "bam_quality_control_and_metrics",
        representative_stage_id: "bam.validate",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "bam.validate",
            proof_pointer: Some("/cases/0/validation_report"),
        },
        parse_kind: ParseKind::BamValidationSummary,
        snapshot_keys: &["stage_id", "validation_report_present"],
    },
    FamilyProbeSpec {
        family_id: "overlap_correction",
        representative_stage_id: "bam.overlap_correction",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "bam.overlap_correction",
            proof_pointer: Some("/overlap_correction_summary"),
        },
        parse_kind: ParseKind::BamOverlapCorrectionSummary,
        snapshot_keys: &["method", "pair_count", "corrected_pairs", "corrected_overlap_bases"],
    },
    FamilyProbeSpec {
        family_id: "variant_analysis_and_recalibration",
        representative_stage_id: "bam.recalibration",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "bam.recalibration",
            proof_pointer: Some("/recalibration_summary"),
        },
        parse_kind: ParseKind::BamRecalibrationSummary,
        snapshot_keys: &["requested_mode", "effective_mode", "status", "coverage_gate"],
    },
    FamilyProbeSpec {
        family_id: "genotyping_and_population_inference",
        representative_stage_id: "bam.genotyping",
        probe_kind: ProbeKind::GeneratedBamGenotyping,
        parse_kind: ParseKind::BamGenotypingJson,
        snapshot_keys: &["producer", "reference", "sites", "call_rate", "mean_posterior"],
    },
    FamilyProbeSpec {
        family_id: "damage_and_postmortem_bias",
        representative_stage_id: "bam.damage",
        probe_kind: ProbeKind::GeneratedBamDamage,
        parse_kind: ParseKind::BamDamageEvidence,
        snapshot_keys: &["damage_signal", "terminal_c_to_t_5p", "terminal_g_to_a_3p"],
    },
    FamilyProbeSpec {
        family_id: "authenticity_assessment",
        representative_stage_id: "bam.authenticity",
        probe_kind: ProbeKind::GeneratedBamAuthenticity,
        parse_kind: ParseKind::BamAuthenticityAdvisory,
        snapshot_keys: &["score", "confidence", "pmd_like_signal_present"],
    },
    FamilyProbeSpec {
        family_id: "contamination_estimation",
        representative_stage_id: "bam.contamination",
        probe_kind: ProbeKind::GeneratedBamContamination,
        parse_kind: ParseKind::BamContaminationJson,
        snapshot_keys: &["method", "estimate", "ci_low", "ci_high"],
    },
    FamilyProbeSpec {
        family_id: "library_complexity_estimation",
        representative_stage_id: "bam.complexity",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "bam.complexity",
            proof_pointer: Some("/complexity_summary"),
        },
        parse_kind: ParseKind::BamComplexitySummary,
        snapshot_keys: &["method", "status", "estimated_distinct_fragments", "observed_fragments"],
    },
    FamilyProbeSpec {
        family_id: "sex_and_haplogroup_inference",
        representative_stage_id: "bam.sex",
        probe_kind: ProbeKind::GeneratedBamSex,
        parse_kind: ParseKind::BamSexJson,
        snapshot_keys: &["method", "x_to_y_ratio", "confidence", "status"],
    },
    FamilyProbeSpec {
        family_id: "kinship_relatedness",
        representative_stage_id: "bam.kinship",
        probe_kind: ProbeKind::GeneratedBamKinship,
        parse_kind: ParseKind::BamKinshipSummary,
        snapshot_keys: &["method", "pair_count", "status", "observed_max_overlap_snps"],
    },
];
