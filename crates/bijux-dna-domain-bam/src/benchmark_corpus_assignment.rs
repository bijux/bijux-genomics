use bijux_dna_core::ids::{StageId, ToolId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkCorpusFamily {
    Corpus01,
    BamMini,
    AdnaBam,
    Genotyping,
    Kinship,
}

impl BenchmarkCorpusFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Corpus01 => "corpus-01",
            Self::BamMini => "corpus-01-bam",
            Self::AdnaBam => "corpus-01-adna-bam",
            Self::Genotyping => "corpus-01-genotyping",
            Self::Kinship => "corpus-01-kinship",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchmarkCorpusAssignment {
    Assigned { family: BenchmarkCorpusFamily, rationale: &'static str },
}

impl BenchmarkCorpusAssignment {
    #[must_use]
    pub const fn assigned_family(&self) -> BenchmarkCorpusFamily {
        match self {
            Self::Assigned { family, .. } => *family,
        }
    }

    #[must_use]
    pub const fn rationale(&self) -> &'static str {
        match self {
            Self::Assigned { rationale, .. } => rationale,
        }
    }
}

#[must_use]
pub fn benchmark_corpus_assignment_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<BenchmarkCorpusAssignment> {
    match (stage_id.as_str(), tool_id.as_str()) {
        ("bam.align", "bwa" | "bowtie2") => Some(BenchmarkCorpusAssignment::Assigned {
            family: BenchmarkCorpusFamily::Corpus01,
            rationale:
                "Alignment consumes governed FASTQ reads, so BAM planning stays on the general corpus-01 read fixture.",
        }),
        ("bam.authenticity", "authenticct" | "damageprofiler" | "pmdtools")
        | (
            "bam.contamination",
            "contammix" | "schmutzi" | "verifybamid2",
        )
        | (
            "bam.damage",
            "addeam" | "damageprofiler" | "mapdamage2" | "ngsbriggs" | "pmdtools" | "pydamage",
        )
        | ("bam.haplogroups", "yleaf")
        | ("bam.sex", "angsd" | "rxy" | "yleaf") => {
            Some(BenchmarkCorpusAssignment::Assigned {
                family: BenchmarkCorpusFamily::AdnaBam,
                rationale:
                    "Ancient-like BAM interpretation requires the governed non-UDG aDNA fixture family rather than the general BAM mini corpus.",
            })
        }
        ("bam.genotyping", "angsd") => Some(BenchmarkCorpusAssignment::Assigned {
            family: BenchmarkCorpusFamily::Genotyping,
            rationale:
                "Genotyping depends on the governed candidate-sites sample, target-regions contract, and shared reference owned by the genotyping BAM corpus.",
        }),
        ("bam.kinship", "angsd" | "king") => Some(BenchmarkCorpusAssignment::Assigned {
            family: BenchmarkCorpusFamily::Kinship,
            rationale:
                "Kinship inference depends on governed low-overlap and related-pair BAM samples plus the shared relatedness panel owned by the kinship corpus.",
        }),
        _ if is_bam_mini_stage_tool(stage_id.as_str(), tool_id.as_str()) => {
            Some(BenchmarkCorpusAssignment::Assigned {
                family: BenchmarkCorpusFamily::BamMini,
                rationale:
                    "General BAM preprocessing, QC, contamination planning, coverage, and haplogroup rows stay on the governed BAM mini corpus.",
            })
        }
        _ => None,
    }
}

#[must_use]
pub fn governed_benchmark_stage_tool_bindings() -> &'static [(&'static str, &'static str)] {
    &[
        ("bam.align", "bowtie2"),
        ("bam.align", "bwa"),
        ("bam.authenticity", "authenticct"),
        ("bam.authenticity", "damageprofiler"),
        ("bam.authenticity", "pmdtools"),
        ("bam.bias_mitigation", "mapdamage2"),
        ("bam.complexity", "preseq"),
        ("bam.contamination", "contammix"),
        ("bam.contamination", "schmutzi"),
        ("bam.contamination", "verifybamid2"),
        ("bam.coverage", "bedtools"),
        ("bam.coverage", "mosdepth"),
        ("bam.coverage", "samtools"),
        ("bam.damage", "addeam"),
        ("bam.damage", "damageprofiler"),
        ("bam.damage", "mapdamage2"),
        ("bam.damage", "ngsbriggs"),
        ("bam.damage", "pmdtools"),
        ("bam.damage", "pydamage"),
        ("bam.duplication_metrics", "picard"),
        ("bam.duplication_metrics", "samtools"),
        ("bam.endogenous_content", "samtools"),
        ("bam.filter", "bamtools"),
        ("bam.filter", "bedtools"),
        ("bam.filter", "samtools"),
        ("bam.gc_bias", "picard"),
        ("bam.genotyping", "angsd"),
        ("bam.haplogroups", "yleaf"),
        ("bam.insert_size", "picard"),
        ("bam.kinship", "angsd"),
        ("bam.kinship", "king"),
        ("bam.length_filter", "picard"),
        ("bam.length_filter", "samtools"),
        ("bam.mapping_summary", "picard"),
        ("bam.mapping_summary", "samtools"),
        ("bam.mapq_filter", "bamtools"),
        ("bam.mapq_filter", "samtools"),
        ("bam.markdup", "picard"),
        ("bam.markdup", "samtools"),
        ("bam.overlap_correction", "bamutil"),
        ("bam.qc_pre", "multiqc"),
        ("bam.qc_pre", "samtools"),
        ("bam.recalibration", "gatk"),
        ("bam.sex", "angsd"),
        ("bam.sex", "rxy"),
        ("bam.sex", "yleaf"),
        ("bam.validate", "bamtools"),
        ("bam.validate", "bedtools"),
        ("bam.validate", "samtools"),
    ]
}

fn is_bam_mini_stage_tool(stage_id: &str, tool_id: &str) -> bool {
    matches!(
        (stage_id, tool_id),
        ("bam.bias_mitigation", "mapdamage2")
            | ("bam.complexity", "preseq")
            | ("bam.coverage", "bedtools" | "mosdepth" | "samtools")
            | ("bam.duplication_metrics", "picard" | "samtools")
            | ("bam.endogenous_content", "samtools")
            | ("bam.filter", "bamtools" | "bedtools" | "samtools")
            | ("bam.gc_bias", "picard")
            | ("bam.insert_size", "picard")
            | ("bam.length_filter", "picard" | "samtools")
            | ("bam.mapping_summary", "picard" | "samtools")
            | ("bam.mapq_filter", "bamtools" | "samtools")
            | ("bam.markdup", "picard" | "samtools")
            | ("bam.overlap_correction", "bamutil")
            | ("bam.qc_pre", "multiqc" | "samtools")
            | ("bam.recalibration", "gatk")
            | ("bam.validate", "bamtools" | "bedtools" | "samtools")
    )
}
