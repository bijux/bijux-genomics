//! Owner: bijux-dna-bench
//! Benchmark-matrix appraisal and hardening-queue workflow ownership.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_appraisal_report, validate_benchmark_matrix_report, validate_hardening_queue_report,
    AppraisalFinding, AppraisalReport, AppraisalSummary, BenchmarkMatrixReport,
    HardeningQueueEntry, HardeningQueueReport, APPRAISAL_REPORT_SCHEMA_V1,
    HARDENING_QUEUE_REPORT_SCHEMA_V1,
};
use serde::{de::DeserializeOwned, Serialize};

trait AppraiserPlugin {
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

/// Appraise every row in a validated benchmark matrix with the registered product appraisers.
///
/// # Errors
/// Returns an error when the source matrix or derived persisted appraisal is invalid.
pub fn appraise_matrix_report(matrix: BenchmarkMatrixReport) -> Result<AppraisalReport> {
    validate_benchmark_matrix_report(&matrix).context("validate appraisal source matrix")?;
    let mut findings = Vec::new();
    for plugin in plugins() {
        findings.extend(plugin.appraise(&matrix));
    }
    findings.sort_by(finding_order);
    let report = AppraisalReport {
        schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
        campaign_id: matrix.campaign_id,
        domain: matrix.domain,
        summary: summarize_findings(&findings),
        findings,
    };
    validate_appraisal_report(&report).context("validate derived appraisal report")?;
    Ok(report)
}

/// Group a validated appraisal into a deterministic hardening queue.
///
/// # Errors
/// Returns an error when the appraisal or derived queue violates its persisted contract.
pub fn hardening_queue_from_appraisal(appraisal: AppraisalReport) -> Result<HardeningQueueReport> {
    validate_appraisal_report(&appraisal).context("validate hardening queue source appraisal")?;
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
    entries.sort_by(queue_order);
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.affected_rows.sort();
        entry.source_appraisers.sort();
        entry.queue_id = format!("hardening-{:04}", index + 1);
    }
    let report = HardeningQueueReport {
        schema_version: HARDENING_QUEUE_REPORT_SCHEMA_V1.to_string(),
        campaign_id: appraisal.campaign_id,
        domain: appraisal.domain,
        entries,
    };
    validate_hardening_queue_report(&report).context("validate derived hardening queue")?;
    Ok(report)
}

/// Load and validate a persisted appraisal report.
///
/// # Errors
/// Returns an error when the file cannot be read, parsed, or admitted.
pub fn load_appraisal_report(path: &Path) -> Result<AppraisalReport> {
    let report = load_json(path)?;
    validate_appraisal_report(&report)
        .with_context(|| format!("validate appraisal report {}", path.display()))?;
    Ok(report)
}

/// Validate and atomically publish an appraisal report.
///
/// # Errors
/// Returns an error when the report is invalid or cannot be written atomically.
pub fn publish_appraisal_report(path: &Path, report: &AppraisalReport) -> Result<()> {
    validate_appraisal_report(report).context("validate appraisal report publication")?;
    publish_json(path, report)
}

/// Load and validate a persisted hardening queue.
///
/// # Errors
/// Returns an error when the file cannot be read, parsed, or admitted.
pub fn load_hardening_queue_report(path: &Path) -> Result<HardeningQueueReport> {
    let report = load_json(path)?;
    validate_hardening_queue_report(&report)
        .with_context(|| format!("validate hardening queue {}", path.display()))?;
    Ok(report)
}

/// Validate and atomically publish a hardening queue.
///
/// # Errors
/// Returns an error when the report is invalid or cannot be written atomically.
pub fn publish_hardening_queue_report(path: &Path, report: &HardeningQueueReport) -> Result<()> {
    validate_hardening_queue_report(report).context("validate hardening queue publication")?;
    publish_json(path, report)
}

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

fn finding_order(left: &AppraisalFinding, right: &AppraisalFinding) -> std::cmp::Ordering {
    severity_weight(&right.severity)
        .cmp(&severity_weight(&left.severity))
        .then_with(|| left.appraiser_id.cmp(&right.appraiser_id))
        .then_with(|| left.row_id.cmp(&right.row_id))
        .then_with(|| left.failure_class.cmp(&right.failure_class))
        .then_with(|| left.result_scope.cmp(&right.result_scope))
}

fn queue_order(left: &HardeningQueueEntry, right: &HardeningQueueEntry) -> std::cmp::Ordering {
    severity_weight(&right.severity)
        .cmp(&severity_weight(&left.severity))
        .then_with(|| left.failure_class.cmp(&right.failure_class))
        .then_with(|| left.recommendation.cmp(&right.recommendation))
}

fn summarize_findings(findings: &[AppraisalFinding]) -> AppraisalSummary {
    let mut by_appraiser = BTreeMap::new();
    let mut by_severity = BTreeMap::new();
    for finding in findings {
        *by_appraiser.entry(finding.appraiser_id.clone()).or_insert(0) += 1;
        *by_severity.entry(finding.severity.clone()).or_insert(0) += 1;
    }
    AppraisalSummary { total_findings: findings.len(), by_appraiser, by_severity }
}

fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))
}

fn publish_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)?;
    bijux_dna_infra::atomic_write_bytes(path, &payload)
        .with_context(|| format!("publish {}", path.display()))
}

impl AppraiserPlugin for RuntimePerformanceAppraiser {
    fn id(&self) -> &'static str {
        "runtime-performance"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut findings = Vec::new();
        for row in &matrix.rows {
            if row.repetitions == 0 {
                findings.push(finding(
                    self.id(),
                    &row.row_id,
                    "critical",
                    "high",
                    "runtime-unrunnable",
                    "encrypted-results",
                    "benchmark row has zero repetitions",
                    "set non-zero repetitions or resolve readiness blockers",
                ));
            } else if row.repetitions < 2 {
                findings.push(finding(
                    self.id(),
                    &row.row_id,
                    "warning",
                    "medium",
                    "runtime-under-sampled",
                    "encrypted-results",
                    "benchmark row has too few repetitions",
                    "increase repetitions to at least 2",
                ));
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
                findings.push(finding(
                    self.id(),
                    &row.row_id,
                    "critical",
                    "high",
                    "missing-tool-binding",
                    "encrypted-results",
                    "stage has no registry-bound tool",
                    "bind stage to at least one governed tool",
                ));
            }
            if !row.image_match.ready {
                findings.push(finding(
                    self.id(),
                    &row.row_id,
                    "warning",
                    "high",
                    "image-mismatch",
                    "encrypted-results",
                    "required tool image is not staged",
                    "prepare or stage matching image before benchmark",
                ));
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
        matrix
            .rows
            .iter()
            .filter(|row| row.readiness.class == "refuse")
            .map(|row| {
                finding(
                    self.id(),
                    &row.row_id,
                    "critical",
                    "high",
                    "scientific-invalidity",
                    "encrypted-results",
                    "row classified as refuse is scientifically invalid",
                    "resolve readiness mismatches before scientific evaluation",
                )
            })
            .collect()
    }
}

impl AppraiserPlugin for ReproducibilityAppraiser {
    fn id(&self) -> &'static str {
        "reproducibility"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        matrix
            .rows
            .iter()
            .filter(|row| row.repetitions < 3 && row.readiness.class == "ready")
            .map(|row| {
                finding(
                    self.id(),
                    &row.row_id,
                    "warning",
                    "medium",
                    "reproducibility-low-repeats",
                    "encrypted-results",
                    "ready row has fewer than 3 repetitions",
                    "set repetitions >= 3 for reproducibility confidence",
                )
            })
            .collect()
    }
}

impl AppraiserPlugin for BackendEquivalenceAppraiser {
    fn id(&self) -> &'static str {
        "backend-equivalence"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        let mut stage_to_tools: BTreeMap<&str, usize> = BTreeMap::new();
        for row in &matrix.rows {
            if row.tool_id != "<unbound>" {
                *stage_to_tools.entry(&row.stage_id).or_insert(0) += 1;
            }
        }
        matrix
            .rows
            .iter()
            .filter(|row| stage_to_tools.get(row.stage_id.as_str()).copied().unwrap_or(0) < 2)
            .map(|row| {
                finding(
                    self.id(),
                    &row.row_id,
                    "warning",
                    "medium",
                    "single-backend",
                    "encrypted-results",
                    "stage has fewer than two backend/tool alternatives",
                    "add alternative backend/tool binding for equivalence checks",
                )
            })
            .collect()
    }
}

impl AppraiserPlugin for FailureClassAppraiser {
    fn id(&self) -> &'static str {
        "failure-class"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        matrix
            .rows
            .iter()
            .filter(|row| row.readiness.class != "ready")
            .map(|row| {
                finding(
                    self.id(),
                    &row.row_id,
                    if row.readiness.class == "refuse" { "critical" } else { "warning" },
                    "high",
                    &format!("readiness-{}", row.readiness.class),
                    "encrypted-results",
                    "row is not fully ready",
                    "resolve readiness reasons and re-run appraisal",
                )
            })
            .collect()
    }
}

impl AppraiserPlugin for CorpusSuitabilityAppraiser {
    fn id(&self) -> &'static str {
        "corpus-suitability"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        matrix
            .rows
            .iter()
            .filter(|row| !row.corpus_match.ready)
            .map(|row| {
                finding(
                    self.id(),
                    &row.row_id,
                    "warning",
                    "medium",
                    "corpus-mismatch",
                    "encrypted-results",
                    "corpus does not match required stage profile",
                    "materialize corpus profile matching stage scientific claim",
                )
            })
            .collect()
    }
}

impl AppraiserPlugin for CodeFreezeAppraiser {
    fn id(&self) -> &'static str {
        "code-freeze"
    }

    fn appraise(&self, matrix: &BenchmarkMatrixReport) -> Vec<AppraisalFinding> {
        matrix
            .rows
            .iter()
            .filter(|row| row.tool_id == "<unbound>" || !row.image_match.ready)
            .map(|row| {
                finding(
                    self.id(),
                    &row.row_id,
                    "warning",
                    "medium",
                    "code-freeze-incomplete",
                    "encrypted-code",
                    "row lacks stable tool/image binding for freeze completeness",
                    "bind tool and image lock before code freeze publication",
                )
            })
            .collect()
    }
}

#[allow(clippy::too_many_arguments)]
fn finding(
    appraiser_id: &str,
    row_id: &str,
    severity: &str,
    confidence: &str,
    failure_class: &str,
    result_scope: &str,
    summary: &str,
    recommendation: &str,
) -> AppraisalFinding {
    AppraisalFinding {
        appraiser_id: appraiser_id.to_string(),
        row_id: row_id.to_string(),
        severity: severity.to_string(),
        confidence: confidence.to_string(),
        failure_class: failure_class.to_string(),
        result_scope: result_scope.to_string(),
        summary: summary.to_string(),
        recommendation: recommendation.to_string(),
    }
}
