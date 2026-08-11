//! Owner: bijux-dna-bench
//! Stable benchmark stage-family declarations.

/// One stable benchmark family used to select and summarize related stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkStageFamily {
    pub family_id: &'static str,
    pub surface_label: &'static str,
    pub stage_ids: &'static [&'static str],
}

pub const BAM_STAGE_FAMILIES: &[BenchmarkStageFamily] = &[
    BenchmarkStageFamily {
        family_id: "bam.align",
        surface_label: "bam.align",
        stage_ids: &["bam.align"],
    },
    BenchmarkStageFamily {
        family_id: "bam.validation_core_qc",
        surface_label: "bam validation and core qc",
        stage_ids: &["bam.validate", "bam.qc_pre", "bam.mapping_summary"],
    },
    BenchmarkStageFamily {
        family_id: "bam.filtering",
        surface_label: "bam filtering",
        stage_ids: &["bam.filter", "bam.mapq_filter", "bam.length_filter"],
    },
    BenchmarkStageFamily {
        family_id: "bam.duplicate_handling",
        surface_label: "bam duplicate handling",
        stage_ids: &["bam.markdup", "bam.duplication_metrics"],
    },
    BenchmarkStageFamily {
        family_id: "bam.complexity",
        surface_label: "bam complexity",
        stage_ids: &["bam.complexity"],
    },
    BenchmarkStageFamily {
        family_id: "bam.coverage",
        surface_label: "bam coverage",
        stage_ids: &["bam.coverage"],
    },
    BenchmarkStageFamily {
        family_id: "bam.insert_size_gc_bias",
        surface_label: "bam insert-size and gc-bias",
        stage_ids: &["bam.insert_size", "bam.gc_bias"],
    },
    BenchmarkStageFamily {
        family_id: "bam.overlap_endogenous_content",
        surface_label: "bam overlap and endogenous-content",
        stage_ids: &["bam.overlap_correction", "bam.endogenous_content"],
    },
    BenchmarkStageFamily {
        family_id: "bam.damage_authenticity",
        surface_label: "bam damage and authenticity",
        stage_ids: &["bam.bias_mitigation", "bam.damage", "bam.authenticity"],
    },
    BenchmarkStageFamily {
        family_id: "bam.contamination_sex_haplogroups",
        surface_label: "bam contamination sex haplogroups",
        stage_ids: &["bam.contamination", "bam.sex", "bam.haplogroups"],
    },
    BenchmarkStageFamily {
        family_id: "bam.recalibration_genotyping",
        surface_label: "bam recalibration and genotyping",
        stage_ids: &["bam.recalibration", "bam.genotyping"],
    },
    BenchmarkStageFamily {
        family_id: "bam.kinship",
        surface_label: "bam kinship",
        stage_ids: &["bam.kinship"],
    },
];

pub const FASTQ_STAGE_FAMILIES: &[BenchmarkStageFamily] = &[
    BenchmarkStageFamily {
        family_id: "fastq.validate_reads",
        surface_label: "fastq.validate_reads",
        stage_ids: &["fastq.validate_reads"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.read_profiling",
        surface_label: "fastq read profiling",
        stage_ids: &[
            "fastq.profile_reads",
            "fastq.profile_read_lengths",
            "fastq.profile_overrepresented_sequences",
        ],
    },
    BenchmarkStageFamily {
        family_id: "fastq.qc_reporting",
        surface_label: "fastq qc reporting",
        stage_ids: &["fastq.report_qc"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.adapter_detection",
        surface_label: "fastq adapter detection",
        stage_ids: &["fastq.detect_adapters"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.trimming",
        surface_label: "fastq trimming",
        stage_ids: &["fastq.trim_reads", "fastq.trim_terminal_damage", "fastq.trim_polyg_tails"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.filtering",
        surface_label: "fastq filtering",
        stage_ids: &["fastq.filter_reads", "fastq.filter_low_complexity"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.duplicate_handling",
        surface_label: "fastq duplicate handling",
        stage_ids: &["fastq.detect_duplicates_premerge", "fastq.remove_duplicates"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.complexity_correction",
        surface_label: "fastq complexity and correction",
        stage_ids: &["fastq.estimate_library_complexity_prealign", "fastq.correct_errors"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.merge_umi",
        surface_label: "fastq merge and umi",
        stage_ids: &["fastq.merge_pairs", "fastq.extract_umis"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.depletion",
        surface_label: "fastq depletion",
        stage_ids: &[
            "fastq.deplete_rrna",
            "fastq.deplete_host",
            "fastq.deplete_reference_contaminants",
        ],
    },
    BenchmarkStageFamily {
        family_id: "fastq.index_reference",
        surface_label: "fastq.index_reference",
        stage_ids: &["fastq.index_reference"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.taxonomy",
        surface_label: "fastq taxonomy",
        stage_ids: &["fastq.screen_taxonomy"],
    },
    BenchmarkStageFamily {
        family_id: "fastq.amplicon",
        surface_label: "fastq amplicon",
        stage_ids: &[
            "fastq.normalize_primers",
            "fastq.remove_chimeras",
            "fastq.infer_asvs",
            "fastq.cluster_otus",
            "fastq.normalize_abundance",
        ],
    },
];

pub const VCF_STAGE_FAMILIES: &[BenchmarkStageFamily] = &[
    BenchmarkStageFamily {
        family_id: "vcf.reference_panel_preparation",
        surface_label: "vcf reference panel preparation",
        stage_ids: &["vcf.prepare_reference_panel"],
    },
    BenchmarkStageFamily {
        family_id: "vcf.calling",
        surface_label: "vcf calling",
        stage_ids: &["vcf.call", "vcf.call_diploid", "vcf.call_gl", "vcf.call_pseudohaploid"],
    },
    BenchmarkStageFamily {
        family_id: "vcf.variant_curation",
        surface_label: "vcf variant curation",
        stage_ids: &["vcf.damage_filter", "vcf.filter", "vcf.gl_propagation", "vcf.postprocess"],
    },
    BenchmarkStageFamily {
        family_id: "vcf.quality_control",
        surface_label: "vcf quality control",
        stage_ids: &["vcf.qc", "vcf.stats"],
    },
    BenchmarkStageFamily {
        family_id: "vcf.phasing",
        surface_label: "vcf phasing",
        stage_ids: &["vcf.phasing"],
    },
    BenchmarkStageFamily {
        family_id: "vcf.imputation",
        surface_label: "vcf imputation",
        stage_ids: &["vcf.impute", "vcf.imputation_metrics"],
    },
    BenchmarkStageFamily {
        family_id: "vcf.population_structure",
        surface_label: "vcf population structure",
        stage_ids: &["vcf.population_structure", "vcf.pca", "vcf.admixture"],
    },
    BenchmarkStageFamily {
        family_id: "vcf.descent_and_demography",
        surface_label: "vcf descent and demography",
        stage_ids: &["vcf.ibd", "vcf.roh", "vcf.demography"],
    },
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{BAM_STAGE_FAMILIES, FASTQ_STAGE_FAMILIES, VCF_STAGE_FAMILIES};

    #[test]
    fn stage_families_have_unique_nonempty_domain_bound_identities() {
        for (domain, families) in [
            ("bam.", BAM_STAGE_FAMILIES),
            ("fastq.", FASTQ_STAGE_FAMILIES),
            ("vcf.", VCF_STAGE_FAMILIES),
        ] {
            let family_ids =
                families.iter().map(|family| family.family_id).collect::<BTreeSet<_>>();
            assert_eq!(family_ids.len(), families.len());
            assert!(families.iter().all(|family| {
                family.family_id.starts_with(domain)
                    && !family.surface_label.is_empty()
                    && !family.stage_ids.is_empty()
                    && family.stage_ids.iter().all(|stage_id| stage_id.starts_with(domain))
            }));
        }
    }

    #[test]
    fn stage_ids_are_assigned_to_one_family_per_domain() {
        for (families, expected_stage_count) in
            [(BAM_STAGE_FAMILIES, 24), (FASTQ_STAGE_FAMILIES, 27), (VCF_STAGE_FAMILIES, 20)]
        {
            let stage_count = families.iter().map(|family| family.stage_ids.len()).sum::<usize>();
            let unique = families
                .iter()
                .flat_map(|family| family.stage_ids.iter().copied())
                .collect::<BTreeSet<_>>();
            assert_eq!(unique.len(), stage_count);
            assert_eq!(stage_count, expected_stage_count);
        }
    }
}
