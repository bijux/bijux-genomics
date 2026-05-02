use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::cli::{AppraiseMatrixArgs, BenchmarkMatrixArgs};
use crate::commands::hpc::{benchmark_matrix, BenchmarkMatrixReport};

const APPRAISAL_SCHEMA_VERSION: &str = "bijux.hpc.appraisal.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalReport {
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub findings: Vec<AppraisalFinding>,
    pub summary: AppraisalSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalFinding {
    pub appraiser_id: String,
    pub row_id: String,
    pub severity: String,
    pub confidence: String,
    pub failure_class: String,
    pub result_scope: String,
    pub summary: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalSummary {
    pub total_findings: usize,
    pub by_appraiser: BTreeMap<String, usize>,
    pub by_severity: BTreeMap<String, usize>,
}

pub trait AppraiserPlugin {
    fn id(&self) -> &'static str;
    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding>;
}

fn plugins() -> Vec<Box<dyn AppraiserPlugin>> {
    Vec::new()
}

fn summarize_findings(findings: &[AppraisalFinding]) -> AppraisalSummary {
    let mut by_appraiser = BTreeMap::new();
    let mut by_severity = BTreeMap::new();
    for finding in findings {
        *by_appraiser.entry(finding.appraiser_id.clone()).or_insert(0) += 1;
        *by_severity.entry(finding.severity.clone()).or_insert(0) += 1;
    }
    AppraisalSummary {
        total_findings: findings.len(),
        by_appraiser,
        by_severity,
    }
}

fn matrix_from_args(args: &AppraiseMatrixArgs) -> Result<BenchmarkMatrixReport> {
    if let Some(path) = &args.matrix {
        let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let value = serde_json::from_str::<BenchmarkMatrixReport>(&raw)
            .with_context(|| format!("parse {}", path.display()))?;
        return Ok(value);
    }
    let Some(config) = args.config.clone() else {
        return Err(anyhow!("appraise-matrix requires either --matrix or --config"));
    };
    benchmark_matrix(&BenchmarkMatrixArgs {
        config,
        env_file: args.env_file.clone(),
        user_overrides: args.user_overrides.clone(),
        domain: args.domain.clone(),
        out: None,
        fail_on_refuse: false,
        json: false,
    })
}

fn write_json_pretty(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(path, &payload)?;
    Ok(())
}

pub fn appraise_matrix(args: &AppraiseMatrixArgs) -> Result<AppraisalReport> {
    let matrix = matrix_from_args(args)?;
    let mut findings = Vec::new();
    for plugin in plugins() {
        findings.extend(plugin.appraise(&matrix));
    }
    findings.sort_by(|left, right| {
        left.appraiser_id
            .cmp(&right.appraiser_id)
            .then_with(|| left.row_id.cmp(&right.row_id))
    });
    let report = AppraisalReport {
        schema_version: APPRAISAL_SCHEMA_VERSION.to_string(),
        campaign_id: matrix.campaign_id,
        domain: matrix.domain,
        summary: summarize_findings(&findings),
        findings,
    };
    if let Some(path) = &args.out {
        write_json_pretty(path, &report)?;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{appraise_matrix, AppraiseMatrixArgs};

    fn write_campaign(root: &std::path::Path) -> std::path::PathBuf {
        for name in [
            "corpora",
            "databases",
            "images",
            "scratch",
            "logs",
            "results",
            "code",
            "imports",
            "baselines",
        ] {
            std::fs::create_dir_all(root.join(name)).expect("create dir");
        }
        std::fs::write(root.join("corpora/general"), b"x").expect("seed corpus");
        std::fs::write(root.join("databases/general"), b"x").expect("seed db");
        std::fs::create_dir_all(root.join("images/apptainer")).expect("seed image dir");
        std::fs::write(root.join("images/apptainer/seqkit.sif"), b"x").expect("seed image");
        let env_path = root.join("campaign.env");
        std::fs::write(&env_path, "BIJUX_SLURM_ACCOUNT=a\nBIJUX_SLURM_PROJECT=p\n").expect("env");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&env_path).expect("env metadata").permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&env_path, perms).expect("env perms");
        }
        let path = root.join("campaign.toml");
        let cfg = format!(
            r#"
[campaign]
id = "appraiser-mini"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_recipients = ["alice"]
env_file = "{root}/campaign.env"

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
            root = root.display()
        );
        std::fs::write(&path, cfg).expect("write config");
        path
    }

    #[test]
    fn appraise_matrix_contract_is_stable() {
        let temp = tempfile::tempdir().expect("temp");
        let config = write_campaign(temp.path());
        let report = appraise_matrix(&AppraiseMatrixArgs {
            matrix: None,
            config: Some(config),
            env_file: None,
            user_overrides: None,
            domain: "all".to_string(),
            out: None,
            json: false,
        })
        .expect("appraise");
        assert_eq!(report.schema_version, "bijux.hpc.appraisal.v1".to_string());
        assert_eq!(report.summary.total_findings, 0);
    }
}
