use anyhow::{anyhow, bail, Result};
use serde::Serialize;

use crate::taxonomy::VcfDomainStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortCardinality {
    One,
    Many,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StagePortContract {
    pub name: &'static str,
    pub data_type: &'static str,
    pub cardinality: PortCardinality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageIoContract {
    pub stage: VcfDomainStage,
    pub inputs: Vec<StagePortContract>,
    pub outputs: Vec<StagePortContract>,
    pub required_inputs: Vec<&'static str>,
    pub required_outputs: Vec<&'static str>,
    pub required_indices: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StageMetricsContract {
    pub stage: VcfDomainStage,
    pub metrics_schema_id: &'static str,
    pub required_metrics: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DamageAwareGenotypeLogicContract {
    pub schema_version: &'static str,
    pub masked_variant_classes: &'static [&'static str],
    pub masking_scope: &'static [&'static str],
    pub provenance_fields: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferencePanelGovernance {
    pub panel_id: String,
    pub reference_build: String,
    pub panel_checksum_sha256: String,
    pub index_checksum_sha256: String,
    pub license_id: String,
    pub license_constraints: Vec<String>,
    pub ancestry_tags: Vec<String>,
    pub target_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PanelSelectionContext {
    pub target_build: String,
    pub ancestry_hint: Option<String>,
    pub use_restricted_license: bool,
}

pub trait PanelSelectionPolicy {
    fn select_panel<'a>(
        &self,
        available: &'a [ReferencePanelGovernance],
        context: &PanelSelectionContext,
    ) -> Option<&'a ReferencePanelGovernance>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPanelSelectionPolicy;

impl PanelSelectionPolicy for DefaultPanelSelectionPolicy {
    fn select_panel<'a>(
        &self,
        available: &'a [ReferencePanelGovernance],
        context: &PanelSelectionContext,
    ) -> Option<&'a ReferencePanelGovernance> {
        available
            .iter()
            .filter(|panel| panel.reference_build == context.target_build)
            .filter(|panel| {
                context.use_restricted_license
                    || !panel
                        .license_constraints
                        .iter()
                        .any(|entry| entry.contains("restricted"))
            })
            .find(|panel| {
                context.ancestry_hint.as_ref().is_none_or(|hint| {
                    panel
                        .ancestry_tags
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case(hint))
                })
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfInvariantState {
    pub sorted_by_contig_and_pos: bool,
    pub bgzip_compressed: bool,
    pub tabix_index_present: bool,
    pub sample_set_consistent: bool,
    pub contig_set_consistent: bool,
}

/// # Errors
/// Returns an error when required VCF invariants are violated for the stage.
pub fn validate_vcf_invariants(stage: VcfDomainStage, state: &VcfInvariantState) -> Result<()> {
    if !state.sorted_by_contig_and_pos {
        bail!("{} requires sorted VCF records", stage.as_str());
    }
    if !state.sample_set_consistent {
        bail!("{} requires sample consistency across inputs", stage.as_str());
    }
    if !state.contig_set_consistent {
        bail!("{} requires contig consistency across inputs", stage.as_str());
    }

    let requires_bgzip = matches!(
        stage,
        VcfDomainStage::Filter
            | VcfDomainStage::GlPropagation
            | VcfDomainStage::Phasing
            | VcfDomainStage::Imputation
            | VcfDomainStage::Impute
            | VcfDomainStage::Postprocess
            | VcfDomainStage::Stats
    );
    if requires_bgzip && !state.bgzip_compressed {
        bail!("{} requires bgzip-compressed VCF", stage.as_str());
    }
    if requires_bgzip && !state.tabix_index_present {
        bail!("{} requires tabix index", stage.as_str());
    }
    Ok(())
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

#[must_use]
pub fn stage_io_contract(stage: VcfDomainStage) -> Option<StageIoContract> {
    fn port(
        name: &'static str,
        data_type: &'static str,
        cardinality: PortCardinality,
    ) -> StagePortContract {
        StagePortContract {
            name,
            data_type,
            cardinality,
        }
    }

    let one = PortCardinality::One;
    let many = PortCardinality::Many;
    let contract = match stage {
        VcfDomainStage::Call | VcfDomainStage::CallDiploid | VcfDomainStage::CallGl | VcfDomainStage::CallPseudohaploid => StageIoContract {
            stage,
            inputs: vec![port("bam", "bam", many)],
            outputs: vec![port("vcf", "vcf", one)],
            required_inputs: vec!["bam"],
            required_outputs: vec!["vcf"],
            required_indices: vec!["bam.bai"],
        },
        VcfDomainStage::DamageFilter
        | VcfDomainStage::Filter
        | VcfDomainStage::GlPropagation
        | VcfDomainStage::Phasing
        | VcfDomainStage::Imputation
        | VcfDomainStage::Impute
        | VcfDomainStage::Postprocess
        | VcfDomainStage::Qc
        | VcfDomainStage::Stats => StageIoContract {
            stage,
            inputs: vec![port("vcf", "vcf", one)],
            outputs: vec![port("vcf_out", "vcf", one)],
            required_inputs: vec!["vcf"],
            required_outputs: vec!["vcf_out"],
            required_indices: vec!["vcf.tbi"],
        },
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Admixture | VcfDomainStage::Roh | VcfDomainStage::Ibd => StageIoContract {
            stage,
            inputs: vec![port("filtered_vcf", "vcf", one)],
            outputs: vec![port("report_json", "json", one)],
            required_inputs: vec!["filtered_vcf"],
            required_outputs: vec!["report_json"],
            required_indices: vec!["vcf.tbi"],
        },
        VcfDomainStage::Demography => StageIoContract {
            stage,
            inputs: vec![port("ibd_segments", "json", one)],
            outputs: vec![port("demography_report", "json", one)],
            required_inputs: vec!["ibd_segments"],
            required_outputs: vec!["demography_report"],
            required_indices: vec![],
        },
        VcfDomainStage::PrepareReferencePanel => StageIoContract {
            stage,
            inputs: vec![port("panel_vcf", "vcf", one)],
            outputs: vec![port("prepared_panel", "vcf", one)],
            required_inputs: vec!["panel_vcf"],
            required_outputs: vec!["prepared_panel"],
            required_indices: vec!["vcf.tbi"],
        },
    };
    Some(contract)
}

#[must_use]
pub fn stage_metrics_contract(stage: VcfDomainStage) -> StageMetricsContract {
    match stage {
        VcfDomainStage::Call => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.call.v1",
            required_metrics: &["variants_called", "snps", "indels"],
        },
        VcfDomainStage::CallDiploid => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.call_diploid.v1",
            required_metrics: &["diploid_called_variants", "het_rate"],
        },
        VcfDomainStage::CallGl => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.call_gl.v1",
            required_metrics: &["gl_sites_count", "mean_gl_depth", "gl_parameter_profile"],
        },
        VcfDomainStage::CallPseudohaploid => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.call_pseudohaploid.v1",
            required_metrics: &["pseudo_called_sites", "pseudo_missing_rate", "pseudo_sampling_seed"],
        },
        VcfDomainStage::DamageFilter => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.damage_filter.v1",
            required_metrics: &["ct_ga_masked_sites", "pmd_filtered_reads", "damage_strategy_applied"],
        },
        VcfDomainStage::Phasing => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.phasing.v1",
            required_metrics: &["switch_error_proxy", "phase_block_n50"],
        },
        VcfDomainStage::Imputation | VcfDomainStage::Impute => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.imputation.v1",
            required_metrics: &["imputed_variant_count", "imputation_info_mean", "rsq_mean"],
        },
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Admixture => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.population_structure.v1",
            required_metrics: &["pc1_variance", "pc2_variance", "cluster_count"],
        },
        VcfDomainStage::Ibd => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.ibd.v1",
            required_metrics: &["ibd_segment_count", "ibd_total_length_cM", "pairwise_ibd_sharing_matrix"],
        },
        VcfDomainStage::Roh => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.roh.v1",
            required_metrics: &["roh_count", "roh_total_mb", "roh_mean_length_mb"],
        },
        VcfDomainStage::Demography => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.demography.v1",
            required_metrics: &["ne_recent", "ne_time_series", "ne_confidence_interval"],
        },
        VcfDomainStage::Qc => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.qc.v1",
            required_metrics: &["missingness_rate", "heterozygosity", "sample_call_rate"],
        },
        VcfDomainStage::Postprocess | VcfDomainStage::PrepareReferencePanel | VcfDomainStage::Filter | VcfDomainStage::GlPropagation | VcfDomainStage::Stats => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.stats.v1",
            required_metrics: &["variants_total", "ti_tv"],
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
        VcfDomainStage::Imputation | VcfDomainStage::Impute => &[
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
        VcfDomainStage::Ibd => &[
            StageFailureMode {
                code: "phase_quality_low",
                meaning: "Phasing quality insufficient for IBD inference",
                triage_hint: "Re-run phasing with stricter QC or higher marker density",
            },
        ],
        VcfDomainStage::Roh => &[
            StageFailureMode {
                code: "roh_call_instability",
                meaning: "ROH segments unstable under current coverage regime",
                triage_hint: "Use diploid regime or adjust ROH thresholds",
            },
        ],
        _ => &[
            StageFailureMode {
                code: "tool_error",
                meaning: "Underlying tool invocation failed",
                triage_hint: "Inspect stderr and command provenance for failing step",
            },
        ],
    }
}

/// # Errors
/// Returns an error if governance records violate required lock metadata.
pub fn validate_reference_panel_governance(panel: &ReferencePanelGovernance) -> Result<()> {
    if panel.panel_id.trim().is_empty() || panel.reference_build.trim().is_empty() {
        return Err(anyhow!("panel governance requires non-empty panel_id/reference_build"));
    }
    if panel.panel_checksum_sha256.len() != 64 || panel.index_checksum_sha256.len() != 64 {
        bail!("panel governance requires 64-char sha256 locks for panel/index");
    }
    if panel.license_id.trim().is_empty() {
        bail!("panel governance requires license_id");
    }
    Ok(())
}
