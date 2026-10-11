use bijux_dna_domain_vcf::VcfDomainStage;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VcfReportPlacement {
    pub(super) section_id: &'static str,
    pub(super) summary_table: &'static str,
}

pub(super) fn placement_for_section(section_id: &str) -> Option<VcfReportPlacement> {
    match section_id {
        "reference_panel_preparation" => Some(VcfReportPlacement {
            section_id: "reference_panel_preparation",
            summary_table: "reference_panel_readiness",
        }),
        "variant_calling" => Some(VcfReportPlacement {
            section_id: "variant_calling",
            summary_table: "variant_calling_metrics",
        }),
        "damage_aware_filtering" => Some(VcfReportPlacement {
            section_id: "damage_aware_filtering",
            summary_table: "damage_filtering_metrics",
        }),
        "quality_control" => Some(VcfReportPlacement {
            section_id: "quality_control",
            summary_table: "quality_control_metrics",
        }),
        "likelihood_postprocess" => Some(VcfReportPlacement {
            section_id: "likelihood_postprocess",
            summary_table: "likelihood_postprocess_metrics",
        }),
        "phasing" => {
            Some(VcfReportPlacement { section_id: "phasing", summary_table: "phasing_metrics" })
        }
        "imputation" => Some(VcfReportPlacement {
            section_id: "imputation",
            summary_table: "imputation_metrics",
        }),
        "normalization" => Some(VcfReportPlacement {
            section_id: "normalization",
            summary_table: "normalization_metrics",
        }),
        "population_structure" => Some(VcfReportPlacement {
            section_id: "population_structure",
            summary_table: "population_structure_metrics",
        }),
        "runs_of_homozygosity" => Some(VcfReportPlacement {
            section_id: "runs_of_homozygosity",
            summary_table: "roh_metrics",
        }),
        "identity_by_descent" => Some(VcfReportPlacement {
            section_id: "identity_by_descent",
            summary_table: "ibd_metrics",
        }),
        "demography" => Some(VcfReportPlacement {
            section_id: "demography",
            summary_table: "demography_metrics",
        }),
        _ => None,
    }
}
pub(super) fn failure_columns_for_stage(stage: VcfDomainStage) -> Vec<String> {
    let mut columns = vec![
        "result_status".to_string(),
        "reason".to_string(),
        "parser_id".to_string(),
        "failure_reason".to_string(),
        "observed_error".to_string(),
    ];
    if matches!(stage, VcfDomainStage::Postprocess | VcfDomainStage::PrepareReferencePanel) {
        columns.push("audit_manifest_path".to_string());
    }
    columns
}
