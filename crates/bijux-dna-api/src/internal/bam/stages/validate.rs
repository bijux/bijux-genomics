use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const LOCAL_VALIDATE_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bam.validate.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalValidateAlignmentFixtureEncoding {
    BinaryBam,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalValidateInputBamIdentity {
    input_bam: String,
    #[serde(default)]
    bam_index: Option<String>,
    #[serde(default)]
    reference_fasta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalValidateSmokeStatus {
    Pass,
    Refusal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalValidateSmokeCaseReport {
    sample_id: String,
    alignment_fixture_encoding: LocalValidateAlignmentFixtureEncoding,
    validation_status: LocalValidateSmokeStatus,
    validation_errors: Vec<String>,
    validation_warnings: Vec<String>,
    expect_pass: bool,
    expectation_matched: bool,
    required_refusal_codes: Vec<String>,
    refusal_codes: Vec<String>,
    validation_report_present: bool,
    input_bam_identity: LocalValidateInputBamIdentity,
    total_reads: Option<u64>,
    mapped_reads: Option<u64>,
    duplicate_reads: Option<u64>,
    validation_report: String,
    flagstat: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalValidateSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: u64,
    all_cases_matched: bool,
    cases: Vec<LocalValidateSmokeCaseReport>,
}

/// Materialize the governed local-smoke `bam.validate` artifacts and summary report.
///
/// The written summary artifact lives at `target/local-smoke/bam.validate/validation.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_validate_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_validate_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("target/local-smoke/bam.validate");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let case_reports = cases
        .iter()
        .map(|case| materialize_local_validate_smoke_case(&repo_root, case))
        .collect::<Result<Vec<_>>>()?;

    let summary = LocalValidateSmokeReport {
        schema_version: LOCAL_VALIDATE_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.validate".to_string(),
        case_count: case_reports.len() as u64,
        all_cases_matched: case_reports.iter().all(|case| case.expectation_matched),
        cases: case_reports,
    };

    let report_path = output_root.join("validation.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

fn materialize_local_validate_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalValidateSmokeCasePlan,
) -> Result<LocalValidateSmokeCaseReport> {
    let case_out_dir = resolve_plan_dir(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let bam = repo_root.join(&case.bam);
    let bam_index = case.bam_index.as_ref().map(|path| repo_root.join(path));
    let reference_fasta = case.reference_fasta.as_ref().map(|path| repo_root.join(path));

    let mut summary = bijux_dna_domain_bam::execute_bam_validation(
        &bam,
        bam_index.as_deref(),
        reference_fasta.as_deref(),
    )?;
    normalize_summary_paths(repo_root, &mut summary);

    let validation_report_path = resolve_output_path(repo_root, &case.plan, "validation_report")?;
    let flagstat_path = resolve_output_path(repo_root, &case.plan, "flagstat")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;

    bijux_dna_infra::atomic_write_json(&validation_report_path, &summary)?;
    bijux_dna_infra::atomic_write_bytes(
        &flagstat_path,
        render_flagstat(&summary.flagstat).as_bytes(),
    )?;
    let expectation_matched = summary.validation_report_present == case.expect_pass
        && case
            .required_refusal_codes
            .iter()
            .all(|code| summary.refusal_codes.iter().any(|observed| observed == code));

    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": "bijux.bam.validate.local_smoke.metrics.v1",
            "stage_id": "bam.validate",
            "sample_id": case.sample_id,
            "alignment_fixture_encoding": match case.alignment_fixture_encoding {
                bijux_dna_planner_bam::stage_api::LocalValidateAlignmentFixtureEncoding::BinaryBam => "binary_bam",
            },
            "validation_status": if summary.validation_report_present { "pass" } else { "refusal" },
            "expectation_matched": expectation_matched,
            "validation_report_present": summary.validation_report_present,
            "validation_errors": summary.refusal_codes,
            "validation_warnings": Vec::<String>::new(),
            "refusal_codes": summary.refusal_codes,
            "input_bam_identity": {
                "input_bam": summary.input_bam,
                "bam_index": summary.bam_index,
                "reference_fasta": summary.reference_fasta,
            },
            "total_reads": summary.flagstat.total_reads,
            "mapped_reads": summary.flagstat.mapped_reads,
            "duplicate_reads": summary.flagstat.duplicate_reads,
        }),
    )?;

    Ok(LocalValidateSmokeCaseReport {
        sample_id: case.sample_id.clone(),
        alignment_fixture_encoding: match case.alignment_fixture_encoding {
            bijux_dna_planner_bam::stage_api::LocalValidateAlignmentFixtureEncoding::BinaryBam => {
                LocalValidateAlignmentFixtureEncoding::BinaryBam
            }
        },
        validation_status: if summary.validation_report_present {
            LocalValidateSmokeStatus::Pass
        } else {
            LocalValidateSmokeStatus::Refusal
        },
        validation_errors: summary.refusal_codes.clone(),
        validation_warnings: Vec::new(),
        expect_pass: case.expect_pass,
        expectation_matched,
        required_refusal_codes: case.required_refusal_codes.clone(),
        refusal_codes: summary.refusal_codes.clone(),
        validation_report_present: summary.validation_report_present,
        input_bam_identity: LocalValidateInputBamIdentity {
            input_bam: summary.input_bam.display().to_string(),
            bam_index: summary.bam_index.as_ref().map(|path| path.display().to_string()),
            reference_fasta: summary
                .reference_fasta
                .as_ref()
                .map(|path| path.display().to_string()),
        },
        total_reads: summary.flagstat.total_reads,
        mapped_reads: summary.flagstat.mapped_reads,
        duplicate_reads: summary.flagstat.duplicate_reads,
        validation_report: path_relative_to_repo(repo_root, &validation_report_path),
        flagstat: path_relative_to_repo(repo_root, &flagstat_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

fn normalize_summary_paths(
    repo_root: &Path,
    summary: &mut bijux_dna_domain_bam::BamValidationSummaryV1,
) {
    summary.input_bam = relative_path(repo_root, &summary.input_bam);
    if let Some(bam_index) = summary.bam_index.as_mut() {
        *bam_index = relative_path(repo_root, bam_index);
    }
    if let Some(reference_fasta) = summary.reference_fasta.as_mut() {
        *reference_fasta = relative_path(repo_root, reference_fasta);
    }
}

fn resolve_output_path(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<PathBuf> {
    let path = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!("bam.validate local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_dir(repo_root, &path))
}

fn render_flagstat(flagstat: &bijux_dna_domain_bam::BamFlagstatCountsV1) -> String {
    let total_reads = flagstat.total_reads.unwrap_or(0);
    let mapped_reads = flagstat.mapped_reads.unwrap_or(0);
    let duplicate_reads = flagstat.duplicate_reads.unwrap_or(0);
    let mapped_fraction = flagstat
        .mapped_fraction
        .map(|value| format!("{:.2}%", value * 100.0))
        .unwrap_or_else(|| "N/A".to_string());
    format!(
        "{total_reads} + 0 in total (QC-passed reads + QC-failed reads)\n\
{mapped_reads} + 0 mapped ({mapped_fraction} : N/A)\n\
{duplicate_reads} + 0 duplicates\n"
    )
}

fn resolve_plan_dir(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    relative_path(repo_root, path).display().to_string()
}
