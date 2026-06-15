use serde::Serialize;

use crate::taxonomy::VcfDomainStage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DamageAwareGenotypeLogicContract {
    pub schema_version: &'static str,
    pub masked_variant_classes: &'static [&'static str],
    pub masking_scope: &'static [&'static str],
    pub provenance_fields: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StageOutputGuarantee {
    pub final_primary_format: &'static str,
    pub requires_bgzip_tabix: bool,
    pub bcf_optional: bool,
    pub deterministic_header_normalization: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StageArtifactContract {
    pub stage: VcfDomainStage,
    pub required_artifacts: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StageFailureMode {
    pub code: &'static str,
    pub meaning: &'static str,
    pub triage_hint: &'static str,
}

pub const DAMAGE_AWARE_GENOTYPE_LOGIC: DamageAwareGenotypeLogicContract =
    DamageAwareGenotypeLogicContract {
        schema_version: "bijux.vcf.damage_aware_genotype_logic.v1",
        masked_variant_classes: &["ct_transition", "ga_transition", "pmd_suspect"],
        masking_scope: &["genotype_likelihoods", "called_genotypes", "site_filter_flags"],
        provenance_fields: &[
            "damage_strategy",
            "ct_ga_mask_threshold",
            "pmd_score_threshold",
            "masked_site_count",
            "masked_site_fraction",
        ],
    };

pub const OUTPUT_GUARANTEE: StageOutputGuarantee = StageOutputGuarantee {
    final_primary_format: "vcf.gz",
    requires_bgzip_tabix: true,
    bcf_optional: true,
    deterministic_header_normalization: true,
};

#[must_use]
pub fn stage_artifact_contract(stage: VcfDomainStage) -> StageArtifactContract {
    match stage {
        VcfDomainStage::PrepareReferencePanel => StageArtifactContract {
            stage,
            required_artifacts: &[
                "panel_overlap_stats.json",
                "panel_lock_resolution.json",
                "provenance.json",
                "checksums.sha256",
                "logs.txt",
            ],
        },
        VcfDomainStage::Phasing => StageArtifactContract {
            stage,
            required_artifacts: &[
                "phasing_qc.json",
                "switch_error_proxy.tsv",
                "provenance.json",
                "checksums.sha256",
                "logs.txt",
            ],
        },
        VcfDomainStage::Impute => StageArtifactContract {
            stage,
            required_artifacts: &[
                "imputation_qc.json",
                "maf_bin_quality.tsv",
                "overlap_stats.json",
                "provenance.json",
                "checksums.sha256",
                "logs.txt",
                "imputation_accept_decision.json",
            ],
        },
        VcfDomainStage::ImputationMetrics => StageArtifactContract {
            stage,
            required_artifacts: &[
                "imputation_metrics.json",
                "imputation_manifest.json",
                "orchestration_manifest.json",
                "provenance.json",
                "checksums.sha256",
                "logs.txt",
                "imputation_accept_decision.json",
            ],
        },
        VcfDomainStage::Postprocess => StageArtifactContract {
            stage,
            required_artifacts: &[
                "normalization_contract.json",
                "header_normalization_report.json",
                "filter_counts.tsv",
                "provenance.json",
                "checksums.sha256",
                "logs.txt",
            ],
        },
        VcfDomainStage::Qc => StageArtifactContract {
            stage,
            required_artifacts: &[
                "qc_summary.json",
                "qc_tables.tsv",
                "qc_histograms.json",
                "provenance.json",
                "checksums.sha256",
                "logs.txt",
            ],
        },
        VcfDomainStage::Filter => StageArtifactContract {
            stage,
            required_artifacts: &[
                "filter_breakdown.json",
                "filter_breakdown.tsv",
                "filter_explain.json",
                "provenance.json",
                "checksums.sha256",
                "logs.txt",
            ],
        },
        VcfDomainStage::Stats => StageArtifactContract {
            stage,
            required_artifacts: &[
                "bcftools_stats.txt",
                "stats.json",
                "provenance.json",
                "checksums.sha256",
                "logs.txt",
            ],
        },
        _ => StageArtifactContract {
            stage,
            required_artifacts: &["provenance.json", "checksums.sha256", "logs.txt"],
        },
    }
}

#[must_use]
pub fn stage_failure_modes(stage: VcfDomainStage) -> &'static [StageFailureMode] {
    match stage {
        VcfDomainStage::Phasing => &[
            StageFailureMode {
                code: "insufficient_markers",
                meaning: "Variant density is too low for reliable phasing",
                triage_hint: "Relax marker filters or skip phasing for this cohort",
            },
            StageFailureMode {
                code: "reference_panel_mismatch",
                meaning: "Panel build/contig set mismatches cohort variants",
                triage_hint: "Align panel build and normalize contigs before phasing",
            },
        ],
        VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute => &[
            StageFailureMode {
                code: "panel_missing",
                meaning: "No compatible reference panel was selected",
                triage_hint: "Provide a panel with matching build and license permissions",
            },
            StageFailureMode {
                code: "low_info",
                meaning: "Imputation quality collapsed (INFO/Rsq too low)",
                triage_hint: "Check panel ancestry fit and pre-imputation QC",
            },
        ],
        VcfDomainStage::Ibd => &[StageFailureMode {
            code: "phase_quality_low",
            meaning: "Phasing quality insufficient for IBD inference",
            triage_hint: "Re-run phasing with stricter QC or higher marker density",
        }],
        VcfDomainStage::Roh => &[StageFailureMode {
            code: "roh_call_instability",
            meaning: "ROH segments unstable under current coverage regime",
            triage_hint: "Use diploid regime or adjust ROH thresholds",
        }],
        _ => &[StageFailureMode {
            code: "tool_error",
            meaning: "Underlying tool invocation failed",
            triage_hint: "Inspect stderr and command provenance for failing step",
        }],
    }
}
