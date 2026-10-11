use super::metadata::BamReportPlacement;
use bijux_dna_core::id_catalog::{
    BAM_ALIGN, BAM_AUTHENTICITY, BAM_BIAS_MITIGATION, BAM_COMPLEXITY, BAM_CONTAMINATION,
    BAM_COVERAGE, BAM_DAMAGE, BAM_DUPLICATION_METRICS, BAM_ENDOGENOUS_CONTENT, BAM_FILTER,
    BAM_GC_BIAS, BAM_GENOTYPING, BAM_HAPLOGROUPS, BAM_INSERT_SIZE, BAM_KINSHIP, BAM_LENGTH_FILTER,
    BAM_MAPPING_SUMMARY, BAM_MAPQ_FILTER, BAM_MARKDUP, BAM_OVERLAP_CORRECTION, BAM_QC_PRE,
    BAM_RECALIBRATION, BAM_SEX, BAM_VALIDATE,
};
pub(super) fn placement_for_stage(stage_id: &str) -> Option<BamReportPlacement> {
    match stage_id {
        BAM_ALIGN | BAM_VALIDATE | BAM_QC_PRE | BAM_MAPPING_SUMMARY => {
            Some(BamReportPlacement {
                section_id: "alignment_intake",
                section_title: "Alignment Intake",
                summary_table_id: "alignment_baseline",
                summary_table_title: "Alignment Baseline",
                report_focus: "alignment provenance, validation status, and intake QC baselines",
            })
        }
        BAM_FILTER | BAM_MAPQ_FILTER | BAM_LENGTH_FILTER | BAM_OVERLAP_CORRECTION => {
            Some(BamReportPlacement {
                section_id: "alignment_refinement",
                section_title: "Alignment Refinement",
                summary_table_id: "filter_retention",
                summary_table_title: "Filter and Retention",
                report_focus:
                    "alignment cleanup, overlap correction, and retained-read policy effects",
            })
        }
        BAM_MARKDUP | BAM_DUPLICATION_METRICS | BAM_COMPLEXITY => Some(BamReportPlacement {
            section_id: "library_complexity",
            section_title: "Library Complexity",
            summary_table_id: "duplicate_complexity",
            summary_table_title: "Duplicate and Complexity",
            report_focus: "duplicate handling, library saturation, and complexity evidence",
        }),
        BAM_COVERAGE | BAM_INSERT_SIZE | BAM_GC_BIAS | BAM_ENDOGENOUS_CONTENT => {
            Some(BamReportPlacement {
                section_id: "coverage_quality",
                section_title: "Coverage and Quality",
                summary_table_id: "coverage_bias_qc",
                summary_table_title: "Coverage, Bias, and QC",
                report_focus:
                    "coverage regime, insert-size behavior, GC bias, and endogenous-signal reporting",
            })
        }
        BAM_DAMAGE | BAM_AUTHENTICITY | BAM_CONTAMINATION => Some(BamReportPlacement {
            section_id: "ancient_signal",
            section_title: "Ancient Signal",
            summary_table_id: "damage_authenticity",
            summary_table_title: "Damage and Authenticity",
            report_focus: "damage evidence, authenticity advisories, and contamination guardrails",
        }),
        BAM_SEX | BAM_HAPLOGROUPS | BAM_KINSHIP => Some(BamReportPlacement {
            section_id: "sample_identity",
            section_title: "Sample Identity",
            summary_table_id: "identity_inference",
            summary_table_title: "Identity and Relatedness",
            report_focus: "sex inference, haplogroup context, and relatedness interpretation",
        }),
        BAM_BIAS_MITIGATION | BAM_RECALIBRATION | BAM_GENOTYPING => {
            Some(BamReportPlacement {
                section_id: "downstream_readiness",
                section_title: "Downstream Readiness",
                summary_table_id: "variant_readiness",
                summary_table_title: "Variant and Bias Readiness",
                report_focus:
                    "bias control, recalibration, and genotyping readiness before downstream inference",
            })
        }
        _ => None,
    }
}
