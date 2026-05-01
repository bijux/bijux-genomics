use crate::types::FastqReadLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FastqStageLayoutPolicy {
    pub stage_id: &'static str,
    pub accepted_layouts: &'static [FastqReadLayout],
}

const NONE: &[FastqReadLayout] = &[];
const PAIRED_END: &[FastqReadLayout] = &[FastqReadLayout::PairedEnd];
const SINGLE_OR_PAIRED: &[FastqReadLayout] =
    &[FastqReadLayout::SingleEnd, FastqReadLayout::PairedEnd];
const SINGLE_PAIRED_OR_MERGED: &[FastqReadLayout] =
    &[FastqReadLayout::SingleEnd, FastqReadLayout::PairedEnd, FastqReadLayout::Merged];

#[must_use]
pub fn declared_input_layouts_for_stage(stage_id: &str) -> Option<FastqStageLayoutPolicy> {
    match stage_id {
        "fastq.index_reference" => Some(FastqStageLayoutPolicy {
            stage_id: "fastq.index_reference",
            accepted_layouts: NONE,
        }),
        "fastq.validate_reads"
        | "fastq.profile_read_lengths"
        | "fastq.detect_adapters"
        | "fastq.profile_reads"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.report_qc"
        | "fastq.screen_taxonomy"
        | "fastq.remove_chimeras"
        | "fastq.infer_asvs"
        | "fastq.cluster_otus" => Some(FastqStageLayoutPolicy {
            stage_id: match stage_id {
                value => match value {
                    "fastq.validate_reads" => "fastq.validate_reads",
                    "fastq.profile_read_lengths" => "fastq.profile_read_lengths",
                    "fastq.detect_adapters" => "fastq.detect_adapters",
                    "fastq.profile_reads" => "fastq.profile_reads",
                    "fastq.profile_overrepresented_sequences" => {
                        "fastq.profile_overrepresented_sequences"
                    }
                    "fastq.report_qc" => "fastq.report_qc",
                    "fastq.screen_taxonomy" => "fastq.screen_taxonomy",
                    "fastq.remove_chimeras" => "fastq.remove_chimeras",
                    "fastq.infer_asvs" => "fastq.infer_asvs",
                    "fastq.cluster_otus" => "fastq.cluster_otus",
                    _ => unreachable!("validated in match arm"),
                },
            },
            accepted_layouts: SINGLE_PAIRED_OR_MERGED,
        }),
        "fastq.trim_terminal_damage"
        | "fastq.normalize_primers"
        | "fastq.trim_polyg_tails"
        | "fastq.trim_reads"
        | "fastq.filter_reads"
        | "fastq.filter_low_complexity"
        | "fastq.correct_errors"
        | "fastq.remove_duplicates"
        | "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.deplete_rrna"
        | "fastq.detect_duplicates_premerge"
        | "fastq.estimate_library_complexity_prealign" => Some(FastqStageLayoutPolicy {
            stage_id: match stage_id {
                "fastq.trim_terminal_damage" => "fastq.trim_terminal_damage",
                "fastq.normalize_primers" => "fastq.normalize_primers",
                "fastq.trim_polyg_tails" => "fastq.trim_polyg_tails",
                "fastq.trim_reads" => "fastq.trim_reads",
                "fastq.filter_reads" => "fastq.filter_reads",
                "fastq.filter_low_complexity" => "fastq.filter_low_complexity",
                "fastq.correct_errors" => "fastq.correct_errors",
                "fastq.remove_duplicates" => "fastq.remove_duplicates",
                "fastq.deplete_host" => "fastq.deplete_host",
                "fastq.deplete_reference_contaminants" => "fastq.deplete_reference_contaminants",
                "fastq.deplete_rrna" => "fastq.deplete_rrna",
                "fastq.detect_duplicates_premerge" => "fastq.detect_duplicates_premerge",
                "fastq.estimate_library_complexity_prealign" => {
                    "fastq.estimate_library_complexity_prealign"
                }
                _ => unreachable!("validated in match arm"),
            },
            accepted_layouts: SINGLE_OR_PAIRED,
        }),
        "fastq.merge_pairs" => Some(FastqStageLayoutPolicy {
            stage_id: "fastq.merge_pairs",
            accepted_layouts: PAIRED_END,
        }),
        "fastq.extract_umis" => Some(FastqStageLayoutPolicy {
            stage_id: "fastq.extract_umis",
            accepted_layouts: PAIRED_END,
        }),
        "fastq.normalize_abundance" => Some(FastqStageLayoutPolicy {
            stage_id: "fastq.normalize_abundance",
            accepted_layouts: NONE,
        }),
        _ => None,
    }
}

#[must_use]
pub fn stage_accepts_input_layout(stage_id: &str, layout: FastqReadLayout) -> bool {
    declared_input_layouts_for_stage(stage_id)
        .is_some_and(|policy| policy.accepted_layouts.contains(&layout))
}

#[cfg(test)]
mod tests {
    use super::{declared_input_layouts_for_stage, stage_accepts_input_layout};
    use crate::types::{FastqReadLayout, FASTQ_DECLARED_READ_LAYOUTS};

    #[test]
    fn every_layout_declared_for_trim_and_qc_boundaries() {
        for stage_id in ["fastq.trim_reads", "fastq.report_qc", "fastq.extract_umis"] {
            let policy = declared_input_layouts_for_stage(stage_id)
                .unwrap_or_else(|| panic!("missing declared layouts for {stage_id}"));
            for layout in FASTQ_DECLARED_READ_LAYOUTS {
                let _ = policy.accepted_layouts.contains(&layout);
            }
        }
    }

    #[test]
    fn merged_layout_is_routed_only_to_reporting_style_stages() {
        assert!(stage_accepts_input_layout("fastq.report_qc", FastqReadLayout::Merged));
        assert!(!stage_accepts_input_layout("fastq.trim_reads", FastqReadLayout::Merged));
    }
}
