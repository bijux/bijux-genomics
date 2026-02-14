use bijux_dna_domain_vcf::{VcfStage, STAGE_CALL, STAGE_FILTER, STAGE_STATS};

#[derive(Debug, Clone, Copy)]
pub struct VcfStageSpec {
    pub stage_id: &'static str,
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
            metrics_schema: "bijux.vcf.call.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
        VcfStageSpec {
            stage_id: "vcf.call_gl",
            metrics_schema: "bijux.vcf.call_gl.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
        VcfStageSpec {
            stage_id: "vcf.call_diploid",
            metrics_schema: "bijux.vcf.call_diploid.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
        VcfStageSpec {
            stage_id: "vcf.call_pseudohaploid",
            metrics_schema: "bijux.vcf.call_pseudohaploid.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
        VcfStageSpec {
            stage_id: "vcf.damage_filter",
            metrics_schema: "bijux.vcf.damage_filter.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
        VcfStageSpec {
            stage_id: "vcf.gl_propagation",
            metrics_schema: "bijux.vcf.gl_propagation.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
        VcfStageSpec {
            stage_id: "vcf.qc",
            metrics_schema: "bijux.vcf.qc.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
        VcfStageSpec {
            stage_id: STAGE_FILTER,
            metrics_schema: "bijux.vcf.filter.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
        VcfStageSpec {
            stage_id: STAGE_STATS,
            metrics_schema: "bijux.vcf.stats.v1",
            smoke_supported: true,
            parser_supported: true,
            experimental: false,
        },
    ]
}

#[must_use]
pub fn vcf_stage_completeness(stage: VcfStage) -> bool {
    vcf_stage_catalog()
        .iter()
        .find(|spec| spec.stage_id == stage.as_str())
        .is_some_and(|spec| spec.smoke_supported && spec.parser_supported)
}

#[must_use]
pub fn supported_vcf_stages() -> Vec<&'static str> {
    vcf_stage_catalog()
        .iter()
        .filter(|spec| spec.smoke_supported && spec.parser_supported)
        .map(|spec| spec.stage_id)
        .collect()
}
