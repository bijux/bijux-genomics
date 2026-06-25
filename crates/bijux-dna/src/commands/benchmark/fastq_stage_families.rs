#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FastqStageFamily {
    pub(crate) family_id: &'static str,
    pub(crate) surface_label: &'static str,
    pub(crate) stage_ids: &'static [&'static str],
}

pub(crate) const FASTQ_STAGE_FAMILIES: &[FastqStageFamily] = &[
    FastqStageFamily {
        family_id: "fastq.validate_reads",
        surface_label: "fastq.validate_reads",
        stage_ids: &["fastq.validate_reads"],
    },
    FastqStageFamily {
        family_id: "fastq.read_profiling",
        surface_label: "fastq read profiling",
        stage_ids: &[
            "fastq.profile_reads",
            "fastq.profile_read_lengths",
            "fastq.profile_overrepresented_sequences",
        ],
    },
    FastqStageFamily {
        family_id: "fastq.qc_reporting",
        surface_label: "fastq qc reporting",
        stage_ids: &["fastq.report_qc"],
    },
    FastqStageFamily {
        family_id: "fastq.adapter_detection",
        surface_label: "fastq adapter detection",
        stage_ids: &["fastq.detect_adapters"],
    },
    FastqStageFamily {
        family_id: "fastq.trimming",
        surface_label: "fastq trimming",
        stage_ids: &["fastq.trim_reads", "fastq.trim_terminal_damage", "fastq.trim_polyg_tails"],
    },
    FastqStageFamily {
        family_id: "fastq.filtering",
        surface_label: "fastq filtering",
        stage_ids: &["fastq.filter_reads", "fastq.filter_low_complexity"],
    },
    FastqStageFamily {
        family_id: "fastq.duplicate_handling",
        surface_label: "fastq duplicate handling",
        stage_ids: &["fastq.detect_duplicates_premerge", "fastq.remove_duplicates"],
    },
    FastqStageFamily {
        family_id: "fastq.complexity_correction",
        surface_label: "fastq complexity and correction",
        stage_ids: &["fastq.estimate_library_complexity_prealign", "fastq.correct_errors"],
    },
    FastqStageFamily {
        family_id: "fastq.merge_umi",
        surface_label: "fastq merge and umi",
        stage_ids: &["fastq.merge_pairs", "fastq.extract_umis"],
    },
    FastqStageFamily {
        family_id: "fastq.depletion",
        surface_label: "fastq depletion",
        stage_ids: &[
            "fastq.deplete_rrna",
            "fastq.deplete_host",
            "fastq.deplete_reference_contaminants",
        ],
    },
    FastqStageFamily {
        family_id: "fastq.index_reference",
        surface_label: "fastq.index_reference",
        stage_ids: &["fastq.index_reference"],
    },
    FastqStageFamily {
        family_id: "fastq.taxonomy",
        surface_label: "fastq taxonomy",
        stage_ids: &["fastq.screen_taxonomy"],
    },
    FastqStageFamily {
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
