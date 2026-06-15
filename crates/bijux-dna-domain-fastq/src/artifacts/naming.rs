use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationArtifactPaths {
    pub report_json: PathBuf,
    pub validated_reads_manifest: PathBuf,
    pub validation_log_r1: PathBuf,
    pub validation_log_r2: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastqTransformArtifactPaths {
    pub reads_r1: PathBuf,
    pub reads_r2: Option<PathBuf>,
    pub report_json: PathBuf,
    pub raw_backend_report: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostDepletionArtifactPaths {
    pub retained_r1: PathBuf,
    pub retained_r2: Option<PathBuf>,
    pub rejected_r1: PathBuf,
    pub rejected_r2: Option<PathBuf>,
    pub report_json: PathBuf,
    pub raw_backend_report: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContaminantDepletionArtifactPaths {
    pub retained_r1: PathBuf,
    pub retained_r2: Option<PathBuf>,
    pub rejected_r1: PathBuf,
    pub rejected_r2: Option<PathBuf>,
    pub report_json: PathBuf,
    pub raw_backend_report: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RrnaDepletionArtifactPaths {
    pub retained_r1: PathBuf,
    pub retained_r2: Option<PathBuf>,
    pub rejected_r1: PathBuf,
    pub rejected_r2: Option<PathBuf>,
    pub report_tsv: PathBuf,
    pub report_json: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QcBundleArtifactPaths {
    pub report_json: PathBuf,
    pub multiqc_report: PathBuf,
    pub multiqc_data_dir: PathBuf,
    pub governed_qc_inputs_manifest: PathBuf,
}

#[must_use]
pub fn validation_artifact_paths(out_dir: &Path, paired: bool) -> ValidationArtifactPaths {
    ValidationArtifactPaths {
        report_json: out_dir.join("validation.json"),
        validated_reads_manifest: out_dir.join("validated_reads_manifest.json"),
        validation_log_r1: out_dir.join("validation_r1.log"),
        validation_log_r2: paired.then(|| out_dir.join("validation_r2.log")),
    }
}

#[must_use]
pub fn trim_artifact_paths(
    out_dir: &Path,
    paired: bool,
    output_name: &str,
    raw_backend_report: Option<PathBuf>,
) -> FastqTransformArtifactPaths {
    FastqTransformArtifactPaths {
        reads_r1: if paired {
            out_dir.join(format!("R1.{output_name}"))
        } else {
            out_dir.join(output_name)
        },
        reads_r2: paired.then(|| out_dir.join(format!("R2.{output_name}"))),
        report_json: out_dir.join("trim_report.json"),
        raw_backend_report,
    }
}

#[must_use]
pub fn umi_artifact_paths(out_dir: &Path, paired: bool) -> FastqTransformArtifactPaths {
    FastqTransformArtifactPaths {
        reads_r1: if paired {
            out_dir.join("umi_tagged_R1.fastq.gz")
        } else {
            out_dir.join("umi_tagged.fastq.gz")
        },
        reads_r2: paired.then(|| out_dir.join("umi_tagged_R2.fastq.gz")),
        report_json: out_dir.join("umi_report.json"),
        raw_backend_report: Some(out_dir.join("umi_tools.extract.log")),
    }
}

#[must_use]
pub fn merge_fastq_artifact_paths(out_dir: &Path) -> FastqTransformArtifactPaths {
    FastqTransformArtifactPaths {
        reads_r1: out_dir.join("merged.fastq.gz"),
        reads_r2: Some(out_dir.join("unmerged_R1.fastq.gz")),
        report_json: out_dir.join("merge_report.json"),
        raw_backend_report: Some(out_dir.join("merge_backend.log")),
    }
}

#[must_use]
pub fn singleton_fastq_artifact_path(out_dir: &Path) -> PathBuf {
    out_dir.join("singleton.fastq.gz")
}

#[must_use]
pub fn rejected_fastq_artifact_paths(out_dir: &Path, paired: bool) -> FastqTransformArtifactPaths {
    FastqTransformArtifactPaths {
        reads_r1: if paired {
            out_dir.join("rejected_R1.fastq.gz")
        } else {
            out_dir.join("rejected.fastq.gz")
        },
        reads_r2: paired.then(|| out_dir.join("rejected_R2.fastq.gz")),
        report_json: out_dir.join("rejected_reads_report.json"),
        raw_backend_report: None,
    }
}

#[must_use]
pub fn corrected_fastq_artifact_paths(out_dir: &Path, paired: bool) -> FastqTransformArtifactPaths {
    FastqTransformArtifactPaths {
        reads_r1: if paired {
            out_dir.join("corrected_R1.fastq.gz")
        } else {
            out_dir.join("corrected.fastq.gz")
        },
        reads_r2: paired.then(|| out_dir.join("corrected_R2.fastq.gz")),
        report_json: out_dir.join("correction_report.json"),
        raw_backend_report: Some(out_dir.join("correction_backend.log")),
    }
}

#[must_use]
pub fn host_depletion_artifact_paths(out_dir: &Path, paired: bool) -> HostDepletionArtifactPaths {
    HostDepletionArtifactPaths {
        retained_r1: if paired {
            out_dir.join("host_depleted_R1.fastq.gz")
        } else {
            out_dir.join("host_depleted.fastq.gz")
        },
        retained_r2: paired.then(|| out_dir.join("host_depleted_R2.fastq.gz")),
        rejected_r1: if paired {
            out_dir.join("removed_host_R1.fastq.gz")
        } else {
            out_dir.join("removed_host.fastq.gz")
        },
        rejected_r2: paired.then(|| out_dir.join("removed_host_R2.fastq.gz")),
        report_json: out_dir.join("host_depletion_report.json"),
        raw_backend_report: out_dir.join("bowtie2.host.metrics.txt"),
    }
}

#[must_use]
pub fn contaminant_depletion_artifact_paths(
    out_dir: &Path,
    paired: bool,
) -> ContaminantDepletionArtifactPaths {
    ContaminantDepletionArtifactPaths {
        retained_r1: if paired {
            out_dir.join("contaminant_screened_R1.fastq.gz")
        } else {
            out_dir.join("contaminant_screened.fastq.gz")
        },
        retained_r2: paired.then(|| out_dir.join("contaminant_screened_R2.fastq.gz")),
        rejected_r1: if paired {
            out_dir.join("removed_contaminant_R1.fastq.gz")
        } else {
            out_dir.join("removed_contaminant.fastq.gz")
        },
        rejected_r2: paired.then(|| out_dir.join("removed_contaminant_R2.fastq.gz")),
        report_json: out_dir.join("contaminant_screen_report.json"),
        raw_backend_report: out_dir.join("bowtie2.contaminant.metrics.txt"),
    }
}

#[must_use]
pub fn rrna_depletion_artifact_paths(out_dir: &Path, paired: bool) -> RrnaDepletionArtifactPaths {
    RrnaDepletionArtifactPaths {
        retained_r1: if paired {
            out_dir.join("rrna_filtered_R1.fastq.gz")
        } else {
            out_dir.join("rrna_filtered.fastq.gz")
        },
        retained_r2: paired.then(|| out_dir.join("rrna_filtered_R2.fastq.gz")),
        rejected_r1: if paired {
            out_dir.join("removed_rrna_R1.fastq.gz")
        } else {
            out_dir.join("removed_rrna.fastq.gz")
        },
        rejected_r2: paired.then(|| out_dir.join("removed_rrna_R2.fastq.gz")),
        report_tsv: out_dir.join("rrna_report.tsv"),
        report_json: out_dir.join("rrna_report.json"),
    }
}

#[must_use]
pub fn qc_bundle_artifact_paths(out_dir: &Path) -> QcBundleArtifactPaths {
    QcBundleArtifactPaths {
        report_json: out_dir.join("report_qc_report.json"),
        multiqc_report: out_dir.join("multiqc_report.html"),
        multiqc_data_dir: out_dir.join("multiqc_data"),
        governed_qc_inputs_manifest: out_dir.join("governed_qc_inputs_manifest.json"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        contaminant_depletion_artifact_paths, corrected_fastq_artifact_paths,
        host_depletion_artifact_paths, merge_fastq_artifact_paths, qc_bundle_artifact_paths,
        rejected_fastq_artifact_paths, rrna_depletion_artifact_paths,
        singleton_fastq_artifact_path, trim_artifact_paths, umi_artifact_paths,
        validation_artifact_paths,
    };
    use std::path::Path;

    #[test]
    fn naming_contracts_cover_core_fastq_roles() {
        let trim = trim_artifact_paths(
            Path::new("out"),
            true,
            "fastp.fastq.gz",
            Some(Path::new("out/trim_report.fastp.json").to_path_buf()),
        );
        assert_eq!(trim.reads_r1, Path::new("out/R1.fastp.fastq.gz"));
        assert_eq!(trim.reads_r2.as_deref(), Some(Path::new("out/R2.fastp.fastq.gz")));
        assert_eq!(trim.report_json, Path::new("out/trim_report.json"));

        let umi = umi_artifact_paths(Path::new("out"), true);
        assert_eq!(umi.reads_r1, Path::new("out/umi_tagged_R1.fastq.gz"));
        assert_eq!(umi.reads_r2.as_deref(), Some(Path::new("out/umi_tagged_R2.fastq.gz")));

        let merge = merge_fastq_artifact_paths(Path::new("out"));
        assert_eq!(merge.reads_r1, Path::new("out/merged.fastq.gz"));
        assert_eq!(merge.reads_r2.as_deref(), Some(Path::new("out/unmerged_R1.fastq.gz")));

        let corrected = corrected_fastq_artifact_paths(Path::new("out"), false);
        assert_eq!(corrected.reads_r1, Path::new("out/corrected.fastq.gz"));

        let rejected = rejected_fastq_artifact_paths(Path::new("out"), true);
        assert_eq!(rejected.reads_r1, Path::new("out/rejected_R1.fastq.gz"));
        assert_eq!(rejected.reads_r2.as_deref(), Some(Path::new("out/rejected_R2.fastq.gz")));

        assert_eq!(
            singleton_fastq_artifact_path(Path::new("out")),
            Path::new("out/singleton.fastq.gz")
        );
    }

    #[test]
    fn naming_contracts_stabilize_reports_and_depletion_outputs() {
        let validation = validation_artifact_paths(Path::new("out"), true);
        assert_eq!(validation.report_json, Path::new("out/validation.json"));
        assert_eq!(
            validation.validated_reads_manifest,
            Path::new("out/validated_reads_manifest.json")
        );

        let host = host_depletion_artifact_paths(Path::new("out"), false);
        assert_eq!(host.retained_r1, Path::new("out/host_depleted.fastq.gz"));
        assert_eq!(host.rejected_r1, Path::new("out/removed_host.fastq.gz"));

        let rrna = rrna_depletion_artifact_paths(Path::new("out"), false);
        assert_eq!(rrna.retained_r1, Path::new("out/rrna_filtered.fastq.gz"));
        assert_eq!(rrna.rejected_r1, Path::new("out/removed_rrna.fastq.gz"));

        let contaminant = contaminant_depletion_artifact_paths(Path::new("out"), false);
        assert_eq!(contaminant.retained_r1, Path::new("out/contaminant_screened.fastq.gz"));
        assert_eq!(contaminant.rejected_r1, Path::new("out/removed_contaminant.fastq.gz"));

        let qc = qc_bundle_artifact_paths(Path::new("out"));
        assert_eq!(qc.report_json, Path::new("out/report_qc_report.json"));
        assert_eq!(
            qc.governed_qc_inputs_manifest,
            Path::new("out/governed_qc_inputs_manifest.json")
        );
    }
}
