//! Owner: bijux-dna-bench-vcf
//! Imputation output layout and raw/parser artifact declarations.
use super::commands::artifact;
use super::{Context, Result, VcfDomainStage, VcfImputationFamilyAdapterArtifact};

pub(super) struct OutputLayout {
    pub(super) imputed_vcf: String,
    pub(super) imputed_vcf_tbi: String,
    pub(super) imputation_qc: String,
    pub(super) imputation_accept: String,
    pub(super) quality_output: String,
    pub(super) quality_tsv: Option<String>,
    pub(super) warnings: Option<String>,
    pub(super) imputation_manifest: String,
    pub(super) orchestration_manifest: Option<String>,
    pub(super) panel_mismatch_diagnostics: Option<String>,
    pub(super) log_output: String,
}

impl OutputLayout {
    pub(super) fn new(stage: VcfDomainStage, output_root: &str) -> Self {
        let imputed_vcf_path = format!("{output_root}/imputed.vcf.gz");
        let imputed_vcf_tbi_path = format!("{imputed_vcf_path}.tbi");
        let imputation_qc_path = format!("{output_root}/imputation_qc.json");
        let imputation_accept_path = format!("{output_root}/imputation_accept.json");
        let quality_output_path = if stage == VcfDomainStage::ImputationMetrics {
            format!("{output_root}/imputation_metrics.json")
        } else {
            imputation_qc_path.clone()
        };
        let quality_tsv_path = if stage == VcfDomainStage::Impute {
            Some(format!("{output_root}/imputation_qc.tsv"))
        } else {
            None
        };
        let warnings_path = if stage == VcfDomainStage::Impute {
            Some(format!("{output_root}/warnings.json"))
        } else {
            None
        };
        let imputation_manifest_path = format!("{output_root}/imputation_manifest.json");
        let orchestration_manifest_path = if stage == VcfDomainStage::ImputationMetrics {
            Some(format!("{output_root}/orchestration_manifest.json"))
        } else {
            None
        };
        let panel_mismatch_diagnostics_path = if stage == VcfDomainStage::Impute {
            Some(format!("{output_root}/panel_mismatch_diagnostics.json"))
        } else {
            None
        };
        let log_output_path = format!("{output_root}/logs.txt");

        Self {
            imputed_vcf: imputed_vcf_path,
            imputed_vcf_tbi: imputed_vcf_tbi_path,
            imputation_qc: imputation_qc_path,
            imputation_accept: imputation_accept_path,
            quality_output: quality_output_path,
            quality_tsv: quality_tsv_path,
            warnings: warnings_path,
            imputation_manifest: imputation_manifest_path,
            orchestration_manifest: orchestration_manifest_path,
            panel_mismatch_diagnostics: panel_mismatch_diagnostics_path,
            log_output: log_output_path,
        }
    }
    pub(super) fn declared(
        &self,
        stage: VcfDomainStage,
        output_root: &str,
    ) -> Result<Vec<VcfImputationFamilyAdapterArtifact>> {
        let Self {
            imputed_vcf: imputed_vcf_path,
            imputed_vcf_tbi: imputed_vcf_tbi_path,
            imputation_qc: imputation_qc_path,
            imputation_accept: imputation_accept_path,
            quality_output: quality_output_path,
            quality_tsv: quality_tsv_path,
            warnings: warnings_path,
            imputation_manifest: imputation_manifest_path,
            orchestration_manifest: orchestration_manifest_path,
            panel_mismatch_diagnostics: panel_mismatch_diagnostics_path,
            log_output: log_output_path,
            ..
        } = self;
        let mut declared_outputs = vec![
            artifact("imputed_vcf", "variant", imputed_vcf_path),
            artifact("imputed_tbi", "index", imputed_vcf_tbi_path),
            artifact("imputation_qc_json", "report_json", imputation_qc_path),
            artifact("imputation_accept_json", "report_json", imputation_accept_path),
            artifact("imputation_manifest_json", "report_json", imputation_manifest_path),
            artifact("logs_txt", "log", log_output_path),
        ];
        if stage == VcfDomainStage::ImputationMetrics {
            declared_outputs.push(artifact(
                "imputation_metrics_json",
                "report_json",
                quality_output_path,
            ));
        }
        if let Some(path) = quality_tsv_path {
            declared_outputs.push(artifact("imputation_qc_tsv", "report_tsv", path));
            declared_outputs.push(artifact(
                "maf_bin_quality_tsv",
                "report_tsv",
                &format!("{output_root}/maf_bins.tsv"),
            ));
            declared_outputs.push(artifact(
                "info_hist_json",
                "report_json",
                &format!("{output_root}/info_hist.json"),
            ));
            declared_outputs.push(artifact(
                "warnings_json",
                "report_json",
                warnings_path.as_deref().context("warnings path for vcf.impute")?,
            ));
            declared_outputs.push(artifact(
                "overlap_stats_json",
                "report_json",
                &format!("{output_root}/overlap_stats.json"),
            ));
            declared_outputs.push(artifact(
                "panel_mismatch_diagnostics_json",
                "report_json",
                panel_mismatch_diagnostics_path
                    .as_deref()
                    .context("panel mismatch path for vcf.impute")?,
            ));
        }
        if let Some(path) = orchestration_manifest_path {
            declared_outputs.push(artifact("orchestration_manifest_json", "report_json", path));
        }

        Ok(declared_outputs)
    }
}

pub(super) fn output_ids(stage: VcfDomainStage) -> (Vec<String>, Vec<String>) {
    let mut raw_output_ids = vec![
        "imputed_vcf".to_string(),
        "imputed_tbi".to_string(),
        "imputation_qc_json".to_string(),
        "imputation_accept_json".to_string(),
        "imputation_manifest_json".to_string(),
        "logs_txt".to_string(),
    ];
    if stage == VcfDomainStage::Impute {
        raw_output_ids.extend([
            "imputation_qc_tsv".to_string(),
            "maf_bin_quality_tsv".to_string(),
            "info_hist_json".to_string(),
            "warnings_json".to_string(),
            "overlap_stats_json".to_string(),
            "panel_mismatch_diagnostics_json".to_string(),
        ]);
    } else {
        raw_output_ids.extend([
            "imputation_metrics_json".to_string(),
            "orchestration_manifest_json".to_string(),
        ]);
    }

    let parser_output_ids = if stage == VcfDomainStage::Impute {
        vec![
            "imputation_qc".to_string(),
            "imputation_accept".to_string(),
            "imputation_manifest".to_string(),
            "panel_mismatch_diagnostics".to_string(),
        ]
    } else {
        vec![
            "imputation_metrics".to_string(),
            "imputation_qc".to_string(),
            "imputation_accept".to_string(),
            "imputation_manifest".to_string(),
            "orchestration_manifest".to_string(),
        ]
    };

    (raw_output_ids, parser_output_ids)
}
