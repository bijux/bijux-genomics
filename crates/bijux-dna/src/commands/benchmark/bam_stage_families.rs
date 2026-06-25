#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BamStageFamily {
    pub(crate) family_id: &'static str,
    pub(crate) surface_label: &'static str,
    pub(crate) stage_ids: &'static [&'static str],
}

pub(crate) const BAM_STAGE_FAMILIES: &[BamStageFamily] = &[
    BamStageFamily {
        family_id: "bam.align",
        surface_label: "bam.align",
        stage_ids: &["bam.align"],
    },
    BamStageFamily {
        family_id: "bam.validation_core_qc",
        surface_label: "bam validation and core qc",
        stage_ids: &["bam.validate", "bam.qc_pre", "bam.mapping_summary"],
    },
    BamStageFamily {
        family_id: "bam.filtering",
        surface_label: "bam filtering",
        stage_ids: &["bam.filter", "bam.mapq_filter", "bam.length_filter"],
    },
    BamStageFamily {
        family_id: "bam.duplicate_handling",
        surface_label: "bam duplicate handling",
        stage_ids: &["bam.markdup", "bam.duplication_metrics"],
    },
    BamStageFamily {
        family_id: "bam.complexity",
        surface_label: "bam complexity",
        stage_ids: &["bam.complexity"],
    },
    BamStageFamily {
        family_id: "bam.coverage",
        surface_label: "bam coverage",
        stage_ids: &["bam.coverage"],
    },
    BamStageFamily {
        family_id: "bam.insert_size_gc_bias",
        surface_label: "bam insert-size and gc-bias",
        stage_ids: &["bam.insert_size", "bam.gc_bias"],
    },
    BamStageFamily {
        family_id: "bam.overlap_endogenous_content",
        surface_label: "bam overlap and endogenous-content",
        stage_ids: &["bam.overlap_correction", "bam.endogenous_content"],
    },
    BamStageFamily {
        family_id: "bam.damage_authenticity",
        surface_label: "bam damage and authenticity",
        stage_ids: &["bam.bias_mitigation", "bam.damage", "bam.authenticity"],
    },
    BamStageFamily {
        family_id: "bam.contamination_sex_haplogroups",
        surface_label: "bam contamination sex haplogroups",
        stage_ids: &["bam.contamination", "bam.sex", "bam.haplogroups"],
    },
    BamStageFamily {
        family_id: "bam.recalibration_genotyping",
        surface_label: "bam recalibration and genotyping",
        stage_ids: &["bam.recalibration", "bam.genotyping"],
    },
    BamStageFamily {
        family_id: "bam.kinship",
        surface_label: "bam kinship",
        stage_ids: &["bam.kinship"],
    },
];
