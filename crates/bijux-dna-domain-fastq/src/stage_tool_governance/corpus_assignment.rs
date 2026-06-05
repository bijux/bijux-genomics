use bijux_dna_core::ids::{StageId, ToolId};

use super::profiles::stage_tool_governance_profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkCorpusFamily {
    Corpus01,
    Corpus02,
    Corpus03,
}

impl BenchmarkCorpusFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Corpus01 => "corpus-01",
            Self::Corpus02 => "corpus-02",
            Self::Corpus03 => "corpus-03",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchmarkCorpusAssignment {
    Assigned { family: BenchmarkCorpusFamily, rationale: &'static str },
    Excluded { reason_code: &'static str, rationale: &'static str },
}

impl BenchmarkCorpusAssignment {
    #[must_use]
    pub const fn assigned_family(&self) -> Option<BenchmarkCorpusFamily> {
        match self {
            Self::Assigned { family, .. } => Some(*family),
            Self::Excluded { .. } => None,
        }
    }

    #[must_use]
    pub const fn exclusion_reason_code(&self) -> Option<&'static str> {
        match self {
            Self::Assigned { .. } => None,
            Self::Excluded { reason_code, .. } => Some(reason_code),
        }
    }

    #[must_use]
    pub const fn rationale(&self) -> &'static str {
        match self {
            Self::Assigned { rationale, .. } | Self::Excluded { rationale, .. } => rationale,
        }
    }
}

#[must_use]
pub fn benchmark_corpus_assignment_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<BenchmarkCorpusAssignment> {
    stage_tool_governance_profile(stage_id, tool_id)?;
    Some(match stage_id.as_str() {
        "fastq.index_reference" => BenchmarkCorpusAssignment::Excluded {
            reason_code: "reference_index_stage_has_no_read_corpus",
            rationale:
                "Reference indexing prepares a governed reference asset and does not consume corpus reads.",
        },
        "fastq.screen_taxonomy" => BenchmarkCorpusAssignment::Assigned {
            family: BenchmarkCorpusFamily::Corpus02,
            rationale:
                "Taxonomy screening requires the governed eDNA mock-community corpus rather than the general shotgun slice.",
        },
        "fastq.normalize_primers"
        | "fastq.remove_chimeras"
        | "fastq.infer_asvs"
        | "fastq.cluster_otus"
        | "fastq.normalize_abundance" => BenchmarkCorpusAssignment::Assigned {
            family: BenchmarkCorpusFamily::Corpus03,
            rationale:
                "Amplicon-oriented stages must run on the governed amplicon corpus to preserve primer and ASV or OTU semantics.",
        },
        "fastq.profile_overrepresented_sequences" => BenchmarkCorpusAssignment::Excluded {
            reason_code: "governed_overrepresented_sequence_fixture_missing",
            rationale:
                "No governed corpus fixture currently owns an overrepresented-sequence expectation table for benchmark comparison.",
        },
        "fastq.report_qc" => BenchmarkCorpusAssignment::Excluded {
            reason_code: "governed_multiqc_bundle_fixture_missing",
            rationale:
                "The governed QC-report bundle is not yet owned by a corpus fixture with reviewer-stable benchmark expectations.",
        },
        "fastq.validate_reads"
        | "fastq.profile_read_lengths"
        | "fastq.detect_adapters"
        | "fastq.detect_duplicates_premerge"
        | "fastq.estimate_library_complexity_prealign"
        | "fastq.trim_terminal_damage"
        | "fastq.trim_polyg_tails"
        | "fastq.trim_reads"
        | "fastq.filter_reads"
        | "fastq.profile_reads"
        | "fastq.deplete_rrna"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.filter_low_complexity"
        | "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.correct_errors"
        | "fastq.extract_umis" => BenchmarkCorpusAssignment::Assigned {
            family: BenchmarkCorpusFamily::Corpus01,
            rationale:
                "General FASTQ preprocessing and screening stages stay on the governed corpus-01 slice for local benchmark comparability.",
        },
        _ => BenchmarkCorpusAssignment::Excluded {
            reason_code: "unclassified_fastq_stage",
            rationale:
                "FASTQ stage is not yet classified in the benchmark corpus routing contract and must not claim a governed corpus family.",
        },
    })
}
