//! Owner: bijux-dna-bench-model
//! Admission validation for benchmark stage reports.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::diagnostics::BenchError;
use crate::model::{
    AdmittedBenchmarkStageReport, BenchmarkComparisonStatus, BenchmarkReportVerdict,
    BenchmarkScientificValue, BenchmarkStageCampaignSpec, BenchmarkStageInterfaceSpec,
    BenchmarkStageReport, BenchmarkToolReport,
};

use super::{BENCHMARK_REPORT_ADMISSION_SCHEMA_V1, BENCHMARK_STAGE_REPORT_SCHEMA_V1};

/// Validate a checksummed report admission envelope.
///
/// # Errors
/// Returns an error if the envelope or its report is incomplete or malformed.
pub fn validate_report_admission(
    admission: &AdmittedBenchmarkStageReport,
) -> Result<(), BenchError> {
    if admission.schema_version != BENCHMARK_REPORT_ADMISSION_SCHEMA_V1 {
        return Err(invalid(format!(
            "benchmark report admission schema mismatch: {}",
            admission.schema_version
        )));
    }
    if admission.report_file_name.trim().is_empty()
        || admission.report_file_name.contains('/')
        || admission.report_file_name.contains('\\')
        || std::path::Path::new(&admission.report_file_name).extension()
            != Some(std::ffi::OsStr::new("json"))
    {
        return Err(invalid("benchmark report admission file name is invalid"));
    }
    validate_digest_identity(&admission.source_sha256)?;
    validate_digest_identity(&admission.report_sha256)?;
    let report_bytes = serde_json::to_vec(&admission.report)
        .map_err(|error| invalid(format!("cannot serialize admitted benchmark report: {error}")))?;
    if admission.report_sha256 != sha256_hex(&report_bytes) {
        return Err(invalid("benchmark report admission digest does not match parsed report"));
    }
    validate_stage_report(&admission.report)
}

/// # Errors
/// Returns an error if a benchmark stage report is not structurally admissible.
pub fn validate_stage_report(report: &BenchmarkStageReport) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_STAGE_REPORT_SCHEMA_V1 {
        return Err(invalid(format!(
            "benchmark stage report schema mismatch: {}",
            report.schema_version
        )));
    }
    for (value, field) in [
        (&report.run_id, "report run_id"),
        (&report.stage_id, "report stage_id"),
        (&report.benchmark_profile_id, "report benchmark_profile_id"),
        (&report.scientific_question, "report scientific_question"),
        (&report.comparison_scope, "report comparison_scope"),
        (&report.accuracy_claim, "report accuracy_claim"),
        (&report.interpretation, "report interpretation"),
    ] {
        require_text(value, field)?;
    }
    validate_text_list(&report.limitations, "report limitations", false)?;
    validate_digest_identity(&report.catalog_sha256)?;
    validate_report_design(report)?;
    validate_report_tools(report)?;
    validate_report_comparability(report)?;
    validate_report_comparison(report)
}

/// # Errors
/// Returns an error if a report drifts from its declared campaign or stage interface.
pub fn validate_stage_report_alignment(
    report: &BenchmarkStageReport,
    campaign: &BenchmarkStageCampaignSpec,
    interface: Option<&BenchmarkStageInterfaceSpec>,
) -> Result<(), BenchError> {
    if report.run_id != campaign.run_id
        || report.stage_id != campaign.stage_id
        || report.evidence_tier != campaign.evidence_tier
        || report.benchmark_profile_id != campaign.profile_id
    {
        return Err(invalid(format!(
            "benchmark stage report identity drifts from campaign: {}",
            report.run_id
        )));
    }
    let design = &report.design;
    if design.runtime != campaign.runtime
        || design.scheduler != campaign.scheduler
        || design.container_only != campaign.container_only
        || design.cache_policy != campaign.cache_policy
        || design.warmup_repetitions != campaign.warmup_repetitions
        || design.measured_repetitions != campaign.measured_repetitions
        || design.tool_closure != campaign.tool_ids
        || design.corpus_ids != campaign.corpus_ids
        || design.corpus_catalog_ids != campaign.corpus_catalog_ids
        || design.reference_catalog_ids != campaign.reference_catalog_ids
        || design.database_catalog_ids != campaign.database_catalog_ids
        || design.input_size_keys != campaign.input_size_keys
        || design.result_artifact != campaign.result_artifact
    {
        return Err(invalid(format!(
            "benchmark stage report design drifts from campaign: {}",
            report.run_id
        )));
    }
    validate_report_interface_alignment(report, interface)
}

fn validate_report_design(report: &BenchmarkStageReport) -> Result<(), BenchError> {
    let design = &report.design;
    if !design.container_only || design.warmup_repetitions < 1 || design.measured_repetitions < 3 {
        return Err(invalid("benchmark stage report execution design is not admissible"));
    }
    validate_text_list(&design.tool_closure, "report tool_closure", false)?;
    validate_text_list(&design.corpus_ids, "report corpus_ids", false)?;
    validate_text_list(&design.corpus_catalog_ids, "report corpus_catalog_ids", false)?;
    validate_text_list(&design.reference_catalog_ids, "report reference_catalog_ids", true)?;
    validate_text_list(&design.database_catalog_ids, "report database_catalog_ids", true)?;
    validate_text_list(&design.input_size_keys, "report input_size_keys", false)?;
    require_text(&design.result_artifact, "report result_artifact")?;
    let thread_tools = design.threads_by_tool.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let closure = design.tool_closure.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if thread_tools != closure || design.threads_by_tool.values().any(|threads| *threads < 1) {
        return Err(invalid("benchmark stage report thread allocation does not cover tools"));
    }
    Ok(())
}

fn validate_report_tools(report: &BenchmarkStageReport) -> Result<(), BenchError> {
    if report.tools.is_empty() {
        return Err(invalid("benchmark stage report contains no tool evidence"));
    }
    let mut tools = BTreeSet::new();
    for tool in &report.tools {
        validate_tool_report(tool, report.design.measured_repetitions)?;
        if !tools.insert(tool.tool.as_str()) {
            return Err(invalid(format!("benchmark stage report duplicates tool: {}", tool.tool)));
        }
    }
    let closure = report.design.tool_closure.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if tools != closure {
        return Err(invalid("benchmark stage report lacks exact tool closure"));
    }
    Ok(())
}

fn validate_tool_report(tool: &BenchmarkToolReport, repetitions: u32) -> Result<(), BenchError> {
    if tool.verdict != BenchmarkReportVerdict::Passed {
        return Err(invalid(format!("benchmark tool report verdict is not passed: {}", tool.tool)));
    }
    for (value, field) in [
        (&tool.tool, "tool report id"),
        (&tool.repository_commit, "tool report repository commit"),
        (&tool.image.name, "tool report image name"),
        (&tool.slurm.job_id, "tool report Slurm job id"),
        (&tool.slurm.host, "tool report Slurm host"),
        (&tool.slurm.elapsed, "tool report Slurm elapsed"),
        (&tool.slurm.allocated_cpus, "tool report allocated CPUs"),
        (&tool.determinism.artifact, "tool report deterministic artifact"),
    ] {
        require_text(value, field)?;
    }
    validate_commit_identity(&tool.repository_commit)?;
    for digest in [
        tool.image.sha256.as_str(),
        tool.source_evidence_sha256.as_str(),
        tool.determinism.sha256.as_str(),
    ] {
        validate_digest_identity(digest)?;
    }
    if tool.image.size_bytes == 0 || !tool.determinism.identical_across_repetitions {
        return Err(invalid(format!("benchmark tool evidence is incomplete: {}", tool.tool)));
    }
    if tool.input_bytes.get("total").copied().unwrap_or_default() == 0 {
        return Err(invalid(format!("benchmark tool input bytes are absent: {}", tool.tool)));
    }
    if tool.performance.is_empty() || tool.scientific_observations.is_empty() {
        return Err(invalid(format!("benchmark tool measurements are absent: {}", tool.tool)));
    }
    for distribution in tool.performance.values() {
        let values = [
            distribution.minimum,
            distribution.maximum,
            distribution.mean,
            distribution.sample_standard_deviation,
        ];
        if values.iter().any(|value| !value.is_finite())
            || distribution.minimum > distribution.mean
            || distribution.mean > distribution.maximum
            || distribution.sample_standard_deviation < 0.0
        {
            return Err(invalid(format!("benchmark tool distribution is invalid: {}", tool.tool)));
        }
    }
    for observations in tool.scientific_observations.values() {
        if observations.len() != repetitions as usize
            || observations.iter().any(|value| {
                matches!(value, BenchmarkScientificValue::Number(number) if !number.is_finite() || *number < 0.0)
                    || matches!(value, BenchmarkScientificValue::Text(text) if text.trim().is_empty())
            })
        {
            return Err(invalid(format!("benchmark scientific observations are invalid: {}", tool.tool)));
        }
    }
    for lock in &tool.input_locks {
        require_text(&lock.id, "benchmark input lock id")?;
        require_text(&lock.kind, "benchmark input lock kind")?;
        require_text(&lock.mount_path, "benchmark input lock mount path")?;
        validate_digest_identity(&lock.sha256)?;
    }
    Ok(())
}

fn validate_report_comparability(report: &BenchmarkStageReport) -> Result<(), BenchError> {
    let commits =
        report.tools.iter().map(|tool| tool.repository_commit.as_str()).collect::<BTreeSet<_>>();
    if commits.len() != 1 {
        return Err(invalid("benchmark report tools do not share one repository commit"));
    }
    if let Some(comparability) = &report.comparability {
        validate_commit_identity(&comparability.repository_commit)?;
        validate_digest_identity(&comparability.input_lock_sha256)?;
        validate_digest_identity(&comparability.parameter_lock_sha256)?;
        if !commits.contains(comparability.repository_commit.as_str()) {
            return Err(invalid("benchmark comparability commit differs from tool evidence"));
        }
    } else if report.tools.iter().any(|tool| tool.input_locks.is_empty()) {
        return Err(invalid("benchmark report lacks comparative or per-tool input locks"));
    }
    Ok(())
}

fn validate_report_comparison(report: &BenchmarkStageReport) -> Result<(), BenchError> {
    let comparison = &report.cross_tool_comparison;
    let expected_verdict = match comparison.status {
        BenchmarkComparisonStatus::NotApplicable | BenchmarkComparisonStatus::WithinTolerance => {
            BenchmarkReportVerdict::Passed
        }
        BenchmarkComparisonStatus::OutsideTolerance => BenchmarkReportVerdict::Failed,
    };
    if report.verdict != expected_verdict {
        return Err(invalid("benchmark report verdict disagrees with comparison status"));
    }
    if matches!(comparison.status, BenchmarkComparisonStatus::NotApplicable)
        != comparison.metrics.is_empty()
    {
        return Err(invalid("benchmark comparison status disagrees with metric presence"));
    }
    for metric in &comparison.metrics {
        require_text(&metric.metric, "benchmark comparison metric")?;
        let finite = [
            metric.minimum,
            metric.maximum,
            metric.absolute_range,
            metric.allowed_range,
            metric.absolute_tolerance,
            metric.relative_tolerance,
        ]
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0);
        if !finite
            || metric.observations.is_empty()
            || metric.observations.values().any(|value| !value.is_finite())
            || metric.within_tolerance != (metric.absolute_range <= metric.allowed_range)
        {
            return Err(invalid(format!(
                "benchmark comparison metric is invalid: {}",
                metric.metric
            )));
        }
    }
    Ok(())
}

fn validate_report_interface_alignment(
    report: &BenchmarkStageReport,
    interface: Option<&BenchmarkStageInterfaceSpec>,
) -> Result<(), BenchError> {
    let Some(interface) = interface else {
        return Ok(());
    };
    let comparability = report.comparability.as_ref().ok_or_else(|| {
        invalid(format!("benchmark report lacks comparability for interface: {}", report.stage_id))
    })?;
    let evidence = comparability.stage_interface.as_ref().ok_or_else(|| {
        invalid(format!("benchmark report lacks stage interface evidence: {}", report.stage_id))
    })?;
    if evidence.accepted_input_contracts != interface.accepted_input_contracts
        || evidence.output_contract != interface.output_contract
        || evidence.normalized_result_contract != interface.normalized_result_contract
        || evidence.downstream_probe.stage_id != interface.downstream_probe.consumer_stage_id
        || evidence.downstream_probe.input_contract != interface.downstream_probe.input_contract
        || evidence.downstream_probe.source != interface.downstream_probe.source
    {
        return Err(invalid(format!(
            "benchmark report stage interface drifts: {}",
            report.stage_id
        )));
    }
    Ok(())
}

fn validate_text_list(values: &[String], field: &str, allow_empty: bool) -> Result<(), BenchError> {
    if values.is_empty() && !allow_empty {
        return Err(invalid(format!("{field} cannot be empty")));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !seen.insert(value.as_str()) {
            return Err(invalid(format!("{field} contains duplicate value: {value}")));
        }
    }
    Ok(())
}

fn validate_commit_identity(value: &str) -> Result<(), BenchError> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(format!("benchmark report contains invalid commit identity: {value}")));
    }
    Ok(())
}

fn validate_digest_identity(value: &str) -> Result<(), BenchError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(format!("benchmark report contains invalid digest identity: {value}")));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return Err(invalid(format!("{field} cannot be empty")));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
