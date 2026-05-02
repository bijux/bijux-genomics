use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::cli::{AppraiseMatrixArgs, BenchmarkMatrixArgs, HardeningQueueArgs};
use crate::commands::hpc::{benchmark_matrix, BenchmarkMatrixReport};

const APPRAISAL_SCHEMA_VERSION: &str = "bijux.hpc.appraisal.v1";
const HARDENING_QUEUE_SCHEMA_VERSION: &str = "bijux.hpc.hardening_queue.v1";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardeningQueueReport {
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub entries: Vec<HardeningQueueEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardeningQueueEntry {
    pub queue_id: String,
    pub severity: String,
    pub failure_class: String,
    pub recommendation: String,
    pub affected_rows: Vec<String>,
    pub source_appraisers: Vec<String>,
}

pub trait AppraiserPlugin {
    fn id(&self) -> &'static str;
    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding>;
}

struct RuntimePerformanceAppraiser;
struct ArtifactValidityAppraiser;
struct ScientificOutputAppraiser;
struct ReproducibilityAppraiser;
struct BackendEquivalenceAppraiser;
struct FailureClassAppraiser;
struct CorpusSuitabilityAppraiser;
struct CodeFreezeAppraiser;

fn plugins() -> Vec<Box<dyn AppraiserPlugin>> {
    vec![
        Box::new(RuntimePerformanceAppraiser),
        Box::new(ArtifactValidityAppraiser),
        Box::new(ScientificOutputAppraiser),
        Box::new(ReproducibilityAppraiser),
        Box::new(BackendEquivalenceAppraiser),
        Box::new(FailureClassAppraiser),
        Box::new(CorpusSuitabilityAppraiser),
        Box::new(CodeFreezeAppraiser),
    ]
}

fn severity_weight(value: &str) -> u8 {
    match value {
        "critical" => 3,
        "warning" => 2,
        _ => 1,
    }
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
        severity_weight(&right.severity)
            .cmp(&severity_weight(&left.severity))
            .then_with(|| left.appraiser_id.cmp(&right.appraiser_id))
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

fn appraisal_from_args(args: &HardeningQueueArgs) -> Result<AppraisalReport> {
    if let Some(path) = &args.appraisal {
        let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let value =
            serde_json::from_str::<AppraisalReport>(&raw).with_context(|| format!("parse {}", path.display()))?;
        return Ok(value);
    }
    appraise_matrix(&AppraiseMatrixArgs {
        matrix: args.matrix.clone(),
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_overrides: args.user_overrides.clone(),
        domain: args.domain.clone(),
        out: None,
        json: false,
    })
}

pub fn generate_hardening_queue(args: &HardeningQueueArgs) -> Result<HardeningQueueReport> {
    let appraisal = appraisal_from_args(args)?;
    let mut grouped: BTreeMap<(String, String, String), HardeningQueueEntry> = BTreeMap::new();
    for finding in &appraisal.findings {
        let key = (
            finding.severity.clone(),
            finding.failure_class.clone(),
            finding.recommendation.clone(),
        );
        let entry = grouped.entry(key).or_insert_with(|| HardeningQueueEntry {
            queue_id: String::new(),
            severity: finding.severity.clone(),
            failure_class: finding.failure_class.clone(),
            recommendation: finding.recommendation.clone(),
            affected_rows: Vec::new(),
            source_appraisers: Vec::new(),
        });
        if !entry.affected_rows.iter().any(|row| row == &finding.row_id) {
            entry.affected_rows.push(finding.row_id.clone());
        }
        if !entry.source_appraisers.iter().any(|id| id == &finding.appraiser_id) {
            entry.source_appraisers.push(finding.appraiser_id.clone());
        }
    }
    let mut entries = grouped.into_values().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        severity_weight(&right.severity)
            .cmp(&severity_weight(&left.severity))
            .then_with(|| left.failure_class.cmp(&right.failure_class))
            .then_with(|| left.recommendation.cmp(&right.recommendation))
    });
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.affected_rows.sort();
        entry.source_appraisers.sort();
        entry.queue_id = format!("hardening-{:04}", index + 1);
    }
    let report = HardeningQueueReport {
        schema_version: HARDENING_QUEUE_SCHEMA_VERSION.to_string(),
        campaign_id: appraisal.campaign_id,
        domain: appraisal.domain,
        entries,
    };
    if let Some(path) = &args.out {
        write_json_pretty(path, &report)?;
    }
    Ok(report)
}

impl AppraiserPlugin for RuntimePerformanceAppraiser {
    fn id(&self) -> &'static str {
        "runtime-performance"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut findings = Vec::new();
        for row in &matrix.rows {
            if row.repetitions == 0 {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "critical".to_string(),
                    confidence: "high".to_string(),
                    failure_class: "runtime-unrunnable".to_string(),
                    result_scope: "encrypted-results".to_string(),
                    summary: "benchmark row has zero repetitions".to_string(),
                    recommendation: "set non-zero repetitions or resolve readiness blockers".to_string(),
                });
            } else if row.repetitions < 2 {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "warning".to_string(),
                    confidence: "medium".to_string(),
                    failure_class: "runtime-under-sampled".to_string(),
                    result_scope: "encrypted-results".to_string(),
                    summary: "benchmark row has too few repetitions".to_string(),
                    recommendation: "increase repetitions to at least 2".to_string(),
                });
            }
        }
        findings
    }
}

impl AppraiserPlugin for ArtifactValidityAppraiser {
    fn id(&self) -> &'static str {
        "artifact-validity"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut findings = Vec::new();
        for row in &matrix.rows {
            if row.tool_id == "<unbound>" {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "critical".to_string(),
                    confidence: "high".to_string(),
                    failure_class: "missing-tool-binding".to_string(),
                    result_scope: "encrypted-results".to_string(),
                    summary: "stage has no registry-bound tool".to_string(),
                    recommendation: "bind stage to at least one governed tool".to_string(),
                });
            }
            if !row.image_match.ready {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "warning".to_string(),
                    confidence: "high".to_string(),
                    failure_class: "image-mismatch".to_string(),
                    result_scope: "encrypted-results".to_string(),
                    summary: "required tool image is not staged".to_string(),
                    recommendation: "prepare or stage matching image before benchmark".to_string(),
                });
            }
        }
        findings
    }
}

impl AppraiserPlugin for ScientificOutputAppraiser {
    fn id(&self) -> &'static str {
        "scientific-output"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut findings = Vec::new();
        for row in &matrix.rows {
            if row.readiness.class == "refuse" {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "critical".to_string(),
                    confidence: "high".to_string(),
                    failure_class: "scientific-invalidity".to_string(),
                    result_scope: "encrypted-results".to_string(),
                    summary: "row classified as refuse is scientifically invalid".to_string(),
                    recommendation: "resolve readiness mismatches before scientific evaluation".to_string(),
                });
            }
        }
        findings
    }
}

impl AppraiserPlugin for ReproducibilityAppraiser {
    fn id(&self) -> &'static str {
        "reproducibility"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut findings = Vec::new();
        for row in &matrix.rows {
            if row.repetitions < 3 && row.readiness.class == "ready" {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "warning".to_string(),
                    confidence: "medium".to_string(),
                    failure_class: "reproducibility-low-repeats".to_string(),
                    result_scope: "encrypted-results".to_string(),
                    summary: "ready row has fewer than 3 repetitions".to_string(),
                    recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
                });
            }
        }
        findings
    }
}

impl AppraiserPlugin for BackendEquivalenceAppraiser {
    fn id(&self) -> &'static str {
        "backend-equivalence"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut stage_to_tools: BTreeMap<String, usize> = BTreeMap::new();
        for row in &matrix.rows {
            if row.tool_id != "<unbound>" {
                *stage_to_tools.entry(row.stage_id.clone()).or_insert(0) += 1;
            }
        }
        let mut findings = Vec::new();
        for row in &matrix.rows {
            let tool_count = stage_to_tools.get(&row.stage_id).copied().unwrap_or(0);
            if tool_count < 2 {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "warning".to_string(),
                    confidence: "medium".to_string(),
                    failure_class: "single-backend".to_string(),
                    result_scope: "encrypted-results".to_string(),
                    summary: "stage has fewer than two backend/tool alternatives".to_string(),
                    recommendation: "add alternative backend/tool binding for equivalence checks".to_string(),
                });
            }
        }
        findings
    }
}

impl AppraiserPlugin for FailureClassAppraiser {
    fn id(&self) -> &'static str {
        "failure-class"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut findings = Vec::new();
        for row in &matrix.rows {
            if row.readiness.class != "ready" {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: if row.readiness.class == "refuse" {
                        "critical".to_string()
                    } else {
                        "warning".to_string()
                    },
                    confidence: "high".to_string(),
                    failure_class: format!("readiness-{}", row.readiness.class),
                    result_scope: "encrypted-results".to_string(),
                    summary: "row is not fully ready".to_string(),
                    recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
                });
            }
        }
        findings
    }
}

impl AppraiserPlugin for CorpusSuitabilityAppraiser {
    fn id(&self) -> &'static str {
        "corpus-suitability"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut findings = Vec::new();
        for row in &matrix.rows {
            if !row.corpus_match.ready {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "warning".to_string(),
                    confidence: "medium".to_string(),
                    failure_class: "corpus-mismatch".to_string(),
                    result_scope: "encrypted-results".to_string(),
                    summary: "corpus does not match required stage profile".to_string(),
                    recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
                });
            }
        }
        findings
    }
}

impl AppraiserPlugin for CodeFreezeAppraiser {
    fn id(&self) -> &'static str {
        "code-freeze"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut findings = Vec::new();
        for row in &matrix.rows {
            if row.tool_id == "<unbound>" || !row.image_match.ready {
                findings.push(AppraisalFinding {
                    appraiser_id: self.id().to_string(),
                    row_id: row.row_id.clone(),
                    severity: "warning".to_string(),
                    confidence: "medium".to_string(),
                    failure_class: "code-freeze-incomplete".to_string(),
                    result_scope: "encrypted-code".to_string(),
                    summary: "row lacks stable tool/image binding for freeze completeness".to_string(),
                    recommendation: "bind tool and image lock before code freeze publication".to_string(),
                });
            }
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{appraise_matrix, generate_hardening_queue, AppraiseMatrixArgs, HardeningQueueArgs};

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

    fn write_matrix(path: &std::path::Path) {
        let matrix = serde_json::json!({
            "schema_version": "bijux.hpc.benchmark_matrix.v1",
            "campaign_id": "matrix-fixture",
            "domain": "fastq",
            "domains": ["fastq"],
            "generated_at": "0",
            "summary": {
                "total_rows": 2,
                "readiness_counts": {"ready": 1, "refuse": 1},
                "domain_counts": {"fastq": 2}
            },
            "rows": [
                {
                    "row_id": "row-refuse",
                    "matrix_domain": "fastq",
                    "stage_id": "fastq.validate_reads",
                    "tool_id": "<unbound>",
                    "corpus_match": {"required_profile": "general", "matched_profile": "<missing>", "ready": false},
                    "database_match": {"required_profile": "not-required", "matched_profile": "not-required", "ready": true},
                    "image_match": {"required_profile": "tool-images", "matched_profile": "<missing>", "ready": false},
                    "readiness": {"class": "refuse", "reasons": ["missing corpus", "missing image"]},
                    "repetitions": 0
                },
                {
                    "row_id": "row-ready-low-repeat",
                    "matrix_domain": "fastq",
                    "stage_id": "fastq.trim_reads",
                    "tool_id": "seqkit_v2",
                    "corpus_match": {"required_profile": "general", "matched_profile": "general", "ready": true},
                    "database_match": {"required_profile": "not-required", "matched_profile": "not-required", "ready": true},
                    "image_match": {"required_profile": "tool-images", "matched_profile": "seqkit", "ready": true},
                    "readiness": {"class": "ready", "reasons": []},
                    "repetitions": 2
                }
            ]
        });
        std::fs::write(path, serde_json::to_vec_pretty(&matrix).expect("json")).expect("matrix");
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
        assert!(report.summary.total_findings > 0);
        assert!(report.summary.by_appraiser.contains_key("runtime-performance"));
    }

    #[test]
    fn hardening_queue_groups_findings() {
        let temp = tempfile::tempdir().expect("temp");
        let config = write_campaign(temp.path());
        let report = generate_hardening_queue(&HardeningQueueArgs {
            appraisal: None,
            matrix: None,
            config: Some(config),
            env_file: None,
            user_overrides: None,
            domain: "all".to_string(),
            out: None,
            json: false,
        })
        .expect("queue");
        assert!(!report.entries.is_empty());
        assert!(report.entries[0].queue_id.starts_with("hardening-"));
    }

    #[test]
    fn appraise_matrix_from_fixture_covers_all_registered_plugins() {
        let temp = tempfile::tempdir().expect("temp");
        let matrix = temp.path().join("matrix.json");
        write_matrix(&matrix);
        let report = appraise_matrix(&AppraiseMatrixArgs {
            matrix: Some(matrix),
            config: None,
            env_file: None,
            user_overrides: None,
            domain: "all".to_string(),
            out: None,
            json: false,
        })
        .expect("appraise");

        for appraiser in [
            "runtime-performance",
            "artifact-validity",
            "scientific-output",
            "reproducibility",
            "backend-equivalence",
            "failure-class",
            "corpus-suitability",
            "code-freeze",
        ] {
            assert!(
                report.summary.by_appraiser.contains_key(appraiser),
                "missing appraiser findings for {appraiser}"
            );
        }
    }
}
