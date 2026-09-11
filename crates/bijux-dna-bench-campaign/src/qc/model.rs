use std::path::PathBuf;
#[derive(Debug, Clone, Copy)]
pub(super) struct ReportQcContributorContract {
    pub(super) stage_id: &'static str,
    pub(super) candidate_tool_ids: &'static [&'static str],
    pub(super) default_tool_id: &'static str,
    pub(super) artifact_id: &'static str,
    pub(super) artifact_role: &'static str,
    pub(super) relative_path: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ReportQcUpstreamStage {
    pub(super) stage_id: &'static str,
    pub(super) tool_id: &'static str,
    pub(super) extra_args: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub(super) struct ReportQcContributorArtifact {
    pub(super) contract: ReportQcContributorContract,
    pub(super) tool_id: String,
    pub(super) path: PathBuf,
}

pub(super) const REPORT_QC_CONTRIBUTORS: [ReportQcContributorContract; 6] = [
    ReportQcContributorContract {
        stage_id: "fastq.validate_reads",
        candidate_tool_ids: &["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
        default_tool_id: "fastqvalidator",
        artifact_id: "validation_report",
        artifact_role: "report_json",
        relative_path: "validation.json",
    },
    ReportQcContributorContract {
        stage_id: "fastq.validate_reads",
        candidate_tool_ids: &["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
        default_tool_id: "fastqvalidator",
        artifact_id: "validated_reads_manifest",
        artifact_role: "summary_json",
        relative_path: "validated_reads_manifest.json",
    },
    ReportQcContributorContract {
        stage_id: "fastq.detect_adapters",
        candidate_tool_ids: &["fastqc"],
        default_tool_id: "fastqc",
        artifact_id: "report_json",
        artifact_role: "report_json",
        relative_path: "adapter_report.json",
    },
    ReportQcContributorContract {
        stage_id: "fastq.detect_adapters",
        candidate_tool_ids: &["fastqc"],
        default_tool_id: "fastqc",
        artifact_id: "adapter_evidence_dir",
        artifact_role: "stage_report",
        relative_path: "fastqc",
    },
    ReportQcContributorContract {
        stage_id: "fastq.profile_reads",
        candidate_tool_ids: &["seqkit_stats", "seqkit", "seqfu"],
        default_tool_id: "seqkit_stats",
        artifact_id: "qc_json",
        artifact_role: "metrics_json",
        relative_path: "qc.json",
    },
    ReportQcContributorContract {
        stage_id: "fastq.profile_read_lengths",
        candidate_tool_ids: &["seqkit_stats"],
        default_tool_id: "seqkit_stats",
        artifact_id: "length_distribution_json",
        artifact_role: "metrics_json",
        relative_path: "length_distribution.json",
    },
];

pub(super) const REPORT_QC_UPSTREAM_STAGES: [ReportQcUpstreamStage; 4] = [
    ReportQcUpstreamStage {
        stage_id: "fastq.validate_reads",
        tool_id: "fastqvalidator",
        extra_args: &[],
    },
    ReportQcUpstreamStage {
        stage_id: "fastq.detect_adapters",
        tool_id: "fastqc",
        extra_args: &["--threads", "1"],
    },
    ReportQcUpstreamStage {
        stage_id: "fastq.profile_reads",
        tool_id: "seqkit_stats",
        extra_args: &["--threads", "1"],
    },
    ReportQcUpstreamStage {
        stage_id: "fastq.profile_read_lengths",
        tool_id: "seqkit_stats",
        extra_args: &["--threads", "1", "--histogram-bins", "100"],
    },
];
