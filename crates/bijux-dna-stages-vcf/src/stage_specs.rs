use bijux_dna_domain_vcf::{VcfDomainStage, VcfStage, STAGE_CALL, STAGE_FILTER_READS, STAGE_STATS};

#[derive(Debug, Clone, Copy)]
pub struct VcfStageSpec {
    pub stage_id: &'static str,
    pub status: &'static str,
    pub default_tool_id: &'static str,
    pub metrics_schema: &'static str,
    pub smoke_supported: bool,
    pub parser_supported: bool,
    pub experimental: bool,
}

#[must_use]
pub fn vcf_stage_catalog() -> &'static [VcfStageSpec] {
    &[
        VcfStageSpec {
            stage_id: STAGE_CALL,
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.call.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.call_gl",
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.call_gl.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.call_diploid",
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.call_diploid.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.call_pseudohaploid",
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.call_pseudohaploid.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.damage_filter",
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.damage_filter.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.gl_propagation",
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.gl_propagation.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.qc",
            status: "supported",
            default_tool_id: "plink2",
            metrics_schema: "bijux.vcf.qc.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.pca",
            status: "supported",
            default_tool_id: "plink2",
            metrics_schema: "bijux.vcf.pca.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.population_structure",
            status: "planned",
            default_tool_id: "plink2",
            metrics_schema: "bijux.vcf.population_structure.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.admixture",
            status: "planned",
            default_tool_id: "plink2",
            metrics_schema: "bijux.vcf.admixture.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.roh",
            status: "planned",
            default_tool_id: "plink2",
            metrics_schema: "bijux.vcf.roh.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.ibd",
            status: "planned",
            default_tool_id: "germline",
            metrics_schema: "bijux.vcf.ibd.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.imputation_metrics",
            status: "supported",
            default_tool_id: "beagle",
            metrics_schema: "bijux.vcf.imputation_metrics.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.impute",
            status: "supported",
            default_tool_id: "beagle",
            metrics_schema: "bijux.vcf.impute.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.phasing",
            status: "supported",
            default_tool_id: "shapeit5",
            metrics_schema: "bijux.vcf.phasing.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.postprocess",
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.postprocess.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.prepare_reference_panel",
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.prepare_reference_panel.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: "vcf.demography",
            status: "planned",
            default_tool_id: "ibdne",
            metrics_schema: "bijux.vcf.demography.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: STAGE_FILTER_READS,
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.filter.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
        VcfStageSpec {
            stage_id: STAGE_STATS,
            status: "supported",
            default_tool_id: "bcftools",
            metrics_schema: "bijux.vcf.stats.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: true,
        },
    ]
}

#[must_use]
pub fn vcf_domain_stage_default_tool_id(stage: VcfDomainStage) -> Option<&'static str> {
    vcf_stage_catalog()
        .iter()
        .find(|spec| spec.stage_id == stage.as_str())
        .map(|spec| spec.default_tool_id)
}

#[must_use]
pub fn vcf_domain_stage_adapter_id(stage: VcfDomainStage) -> Option<&'static str> {
    Some(match stage {
        VcfDomainStage::PrepareReferencePanel => "vcf.adapter.reference_panel",
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => "vcf.adapter.calling",
        VcfDomainStage::DamageFilter
        | VcfDomainStage::Filter
        | VcfDomainStage::GlPropagation
        | VcfDomainStage::Postprocess => "vcf.adapter.transform",
        VcfDomainStage::Phasing | VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute => {
            "vcf.adapter.panel_workflow"
        }
        VcfDomainStage::Qc | VcfDomainStage::Stats => "vcf.adapter.quality_control",
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Admixture => {
            "vcf.adapter.population_structure"
        }
        VcfDomainStage::Roh => "vcf.adapter.roh",
        VcfDomainStage::Ibd => "vcf.adapter.ibd",
        VcfDomainStage::Demography => "vcf.adapter.demography",
    })
}

#[must_use]
pub fn vcf_domain_stage_parser_id(stage: VcfDomainStage) -> Option<&'static str> {
    Some(match stage {
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => "vcf.parser.call_summary",
        VcfDomainStage::DamageFilter
        | VcfDomainStage::Filter
        | VcfDomainStage::GlPropagation
        | VcfDomainStage::Phasing
        | VcfDomainStage::Impute
        | VcfDomainStage::Postprocess
        | VcfDomainStage::PrepareReferencePanel => "vcf.parser.vcf_output",
        VcfDomainStage::ImputationMetrics => "vcf.parser.report_json",
        VcfDomainStage::Qc => "vcf.parser.qc_report",
        VcfDomainStage::Stats => "vcf.parser.stats_report",
        VcfDomainStage::PopulationStructure
        | VcfDomainStage::Pca
        | VcfDomainStage::Admixture
        | VcfDomainStage::Roh
        | VcfDomainStage::Ibd
        | VcfDomainStage::Demography => "vcf.parser.report_json",
    })
}

#[must_use]
pub fn vcf_domain_stage_expected_output_ids(
    stage: VcfDomainStage,
) -> Option<&'static [&'static str]> {
    Some(match stage {
        VcfDomainStage::PrepareReferencePanel => &["prepared_panel", "chunks_json"],
        VcfDomainStage::Call => &["called_vcf"],
        VcfDomainStage::CallDiploid => &["diploid_vcf"],
        VcfDomainStage::CallGl => &["gl_sites_vcf"],
        VcfDomainStage::CallPseudohaploid => &["pseudohaploid_vcf"],
        VcfDomainStage::DamageFilter => &["damage_filtered_vcf"],
        VcfDomainStage::Filter => &["filtered_vcf"],
        VcfDomainStage::GlPropagation => &["gl_propagated_vcf"],
        VcfDomainStage::Qc => &["qc_report"],
        VcfDomainStage::Phasing => &["phased_vcf"],
        VcfDomainStage::ImputationMetrics => &["imputation_metrics_json"],
        VcfDomainStage::Impute => &["imputed_vcf"],
        VcfDomainStage::Postprocess => &["postprocess_vcf"],
        VcfDomainStage::PopulationStructure => &["population_structure_report"],
        VcfDomainStage::Pca => &["pca_report"],
        VcfDomainStage::Admixture => &["admixture_report"],
        VcfDomainStage::Roh => &["roh_report"],
        VcfDomainStage::Ibd => &["ibd_segments"],
        VcfDomainStage::Demography => &["demography_report"],
        VcfDomainStage::Stats => &["stats_json"],
    })
}

#[must_use]
pub fn vcf_stage_completeness(stage: VcfStage) -> bool {
    vcf_stage_catalog().iter().find(|spec| spec.stage_id == stage.as_str()).is_some_and(|spec| {
        spec.status == "supported" && spec.smoke_supported && spec.parser_supported
    })
}

#[must_use]
pub fn vcf_domain_stage_completeness(stage: VcfDomainStage) -> bool {
    vcf_stage_catalog().iter().find(|spec| spec.stage_id == stage.as_str()).is_some_and(|spec| {
        spec.status == "supported" && spec.smoke_supported && spec.parser_supported
    })
}

#[must_use]
pub fn supported_vcf_stages() -> Vec<&'static str> {
    vcf_stage_catalog()
        .iter()
        .filter(|spec| spec.status == "supported" && spec.smoke_supported && spec.parser_supported)
        .map(|spec| spec.stage_id)
        .collect()
}
