#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VcfStageFamily {
    pub(crate) family_id: &'static str,
    pub(crate) surface_label: &'static str,
    pub(crate) stage_ids: &'static [&'static str],
}

pub(crate) const VCF_STAGE_FAMILIES: &[VcfStageFamily] = &[
    VcfStageFamily {
        family_id: "vcf.reference_panel_preparation",
        surface_label: "vcf reference panel preparation",
        stage_ids: &["vcf.prepare_reference_panel"],
    },
    VcfStageFamily {
        family_id: "vcf.calling",
        surface_label: "vcf calling",
        stage_ids: &["vcf.call", "vcf.call_diploid", "vcf.call_gl", "vcf.call_pseudohaploid"],
    },
    VcfStageFamily {
        family_id: "vcf.variant_curation",
        surface_label: "vcf variant curation",
        stage_ids: &["vcf.damage_filter", "vcf.filter", "vcf.gl_propagation", "vcf.postprocess"],
    },
    VcfStageFamily {
        family_id: "vcf.quality_control",
        surface_label: "vcf quality control",
        stage_ids: &["vcf.qc", "vcf.stats"],
    },
    VcfStageFamily {
        family_id: "vcf.phasing",
        surface_label: "vcf phasing",
        stage_ids: &["vcf.phasing"],
    },
    VcfStageFamily {
        family_id: "vcf.imputation",
        surface_label: "vcf imputation",
        stage_ids: &["vcf.impute", "vcf.imputation_metrics"],
    },
    VcfStageFamily {
        family_id: "vcf.population_structure",
        surface_label: "vcf population structure",
        stage_ids: &["vcf.population_structure", "vcf.pca", "vcf.admixture"],
    },
    VcfStageFamily {
        family_id: "vcf.descent_and_demography",
        surface_label: "vcf descent and demography",
        stage_ids: &["vcf.ibd", "vcf.roh", "vcf.demography"],
    },
];
