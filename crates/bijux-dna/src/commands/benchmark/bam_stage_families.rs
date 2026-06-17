#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BamStageFamily {
    pub(crate) goal_id: u32,
    pub(crate) family_id: &'static str,
    pub(crate) surface_label: &'static str,
    pub(crate) stage_ids: &'static [&'static str],
}

pub(crate) const BAM_STAGE_FAMILIES: &[BamStageFamily] = &[
    BamStageFamily {
        goal_id: 380,
        family_id: "bam.align",
        surface_label: "bam.align",
        stage_ids: &["bam.align"],
    },
    BamStageFamily {
        goal_id: 381,
        family_id: "bam.validation_core_qc",
        surface_label: "bam validation and core qc",
        stage_ids: &["bam.validate", "bam.qc_pre", "bam.mapping_summary"],
    },
    BamStageFamily {
        goal_id: 382,
        family_id: "bam.filtering",
        surface_label: "bam filtering",
        stage_ids: &["bam.filter", "bam.mapq_filter", "bam.length_filter"],
    },
    BamStageFamily {
        goal_id: 383,
        family_id: "bam.duplicate_handling",
        surface_label: "bam duplicate handling",
        stage_ids: &["bam.markdup", "bam.duplication_metrics"],
    },
    BamStageFamily {
        goal_id: 384,
        family_id: "bam.complexity",
        surface_label: "bam complexity",
        stage_ids: &["bam.complexity"],
    },
    BamStageFamily {
        goal_id: 385,
        family_id: "bam.coverage",
        surface_label: "bam coverage",
        stage_ids: &["bam.coverage"],
    },
    BamStageFamily {
        goal_id: 386,
        family_id: "bam.insert_size_gc_bias",
        surface_label: "bam insert-size and gc-bias",
        stage_ids: &["bam.insert_size", "bam.gc_bias"],
    },
    BamStageFamily {
        goal_id: 387,
        family_id: "bam.overlap_endogenous_content",
        surface_label: "bam overlap and endogenous-content",
        stage_ids: &["bam.overlap_correction", "bam.endogenous_content"],
    },
    BamStageFamily {
        goal_id: 388,
        family_id: "bam.damage_authenticity",
        surface_label: "bam damage and authenticity",
        stage_ids: &["bam.bias_mitigation", "bam.damage", "bam.authenticity"],
    },
    BamStageFamily {
        goal_id: 389,
        family_id: "bam.contamination_sex_haplogroups",
        surface_label: "bam contamination sex haplogroups",
        stage_ids: &["bam.contamination", "bam.sex", "bam.haplogroups"],
    },
    BamStageFamily {
        goal_id: 390,
        family_id: "bam.recalibration_genotyping",
        surface_label: "bam recalibration and genotyping",
        stage_ids: &["bam.recalibration", "bam.genotyping"],
    },
    BamStageFamily {
        goal_id: 391,
        family_id: "bam.kinship",
        surface_label: "bam kinship",
        stage_ids: &["bam.kinship"],
    },
];
