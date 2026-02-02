use crate::BamStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditArtifact {
    pub name: &'static str,
    pub filename: &'static str,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn required_audit_artifacts(stage: BamStage) -> &'static [AuditArtifact] {
    match stage {
        BamStage::Validate => &[
            AuditArtifact {
                name: "validation_report",
                filename: "validation.json",
            },
            AuditArtifact {
                name: "flagstat",
                filename: "flagstat.txt",
            },
        ],
        BamStage::QcPre => &[
            AuditArtifact {
                name: "qc_report",
                filename: "qc_pre.json",
            },
            AuditArtifact {
                name: "flagstat",
                filename: "flagstat.txt",
            },
            AuditArtifact {
                name: "stats",
                filename: "samtools_stats.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "qc_pre.summary.json",
            },
        ],
        BamStage::Filter => &[
            AuditArtifact {
                name: "filtered_bam",
                filename: "filtered.bam",
            },
            AuditArtifact {
                name: "filtered_bai",
                filename: "filtered.bam.bai",
            },
            AuditArtifact {
                name: "flagstat",
                filename: "flagstat.txt",
            },
            AuditArtifact {
                name: "idxstats",
                filename: "idxstats.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "filter.summary.json",
            },
        ],
        BamStage::Markdup => &[
            AuditArtifact {
                name: "markdup_bam",
                filename: "markdup.bam",
            },
            AuditArtifact {
                name: "markdup_bai",
                filename: "markdup.bam.bai",
            },
            AuditArtifact {
                name: "flagstat",
                filename: "flagstat.txt",
            },
            AuditArtifact {
                name: "idxstats",
                filename: "idxstats.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "markdup.summary.json",
            },
        ],
        BamStage::Complexity => &[
            AuditArtifact {
                name: "complexity_report",
                filename: "complexity.json",
            },
            AuditArtifact {
                name: "preseq",
                filename: "preseq.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "complexity.summary.json",
            },
        ],
        BamStage::Coverage => &[
            AuditArtifact {
                name: "coverage_report",
                filename: "coverage.json",
            },
            AuditArtifact {
                name: "coverage_summary",
                filename: "coverage.mosdepth.summary.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "coverage.summary.json",
            },
        ],
        BamStage::Damage => &[
            AuditArtifact {
                name: "damage_report",
                filename: "damage.json",
            },
            AuditArtifact {
                name: "damage_pydamage",
                filename: "damage.pydamage.json",
            },
            AuditArtifact {
                name: "damage_profiler",
                filename: "damage.profiler.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "damage.summary.json",
            },
        ],
        BamStage::Authenticity => &[
            AuditArtifact {
                name: "authenticity_report",
                filename: "authenticity.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "authenticity.summary.json",
            },
        ],
        BamStage::Contamination => &[
            AuditArtifact {
                name: "contamination_report",
                filename: "contamination.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "contamination.summary.json",
            },
        ],
        BamStage::Sex => &[
            AuditArtifact {
                name: "sex_report",
                filename: "sex.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "sex.summary.json",
            },
        ],
        BamStage::BiasMitigation => &[
            AuditArtifact {
                name: "bias_report",
                filename: "bias_mitigation.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "bias_mitigation.summary.json",
            },
        ],
        BamStage::Recalibration => &[
            AuditArtifact {
                name: "recal_bam",
                filename: "recal.bam",
            },
            AuditArtifact {
                name: "recal_bai",
                filename: "recal.bam.bai",
            },
            AuditArtifact {
                name: "recal_report",
                filename: "recalibration.report.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "recalibration.summary.json",
            },
        ],
        BamStage::Haplogroups => &[
            AuditArtifact {
                name: "haplogroups",
                filename: "haplogroups.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "haplogroups.summary.json",
            },
        ],
        BamStage::Genotyping => &[
            AuditArtifact {
                name: "genotyping_report",
                filename: "genotyping.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "genotyping.summary.json",
            },
        ],
        BamStage::Kinship => &[
            AuditArtifact {
                name: "kinship_report",
                filename: "kinship.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "kinship.summary.json",
            },
        ],
    }
}
