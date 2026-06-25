#![cfg_attr(test, allow(clippy::expect_used))]

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_stage_commands::materialize_local_stage;

pub(crate) const FASTQ_DUPLICATES_TRUTH_FIXTURE_ID: &str = "fastq-duplicates-truth";
pub(crate) const FASTQ_DUPLICATES_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_duplicates_truth.v1";
const FASTQ_DUPLICATES_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_duplicates_truth.expected.v1";
const FASTQ_DUPLICATES_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_duplicates_truth.validation.v1";
const REQUIRED_STAGE_IDS: &[&str] =
    &["fastq.detect_duplicates_premerge", "fastq.remove_duplicates"];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastqDuplicatesTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    stage_ids: Vec<String>,
    source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqDuplicatesTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_stage_count: usize,
    pub(crate) validated_case_count: usize,
    pub(crate) valid: bool,
    pub(crate) checked_cases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct FastqDuplicatesTruthBundle {
    schema_version: String,
    fixture_id: String,
    stage_truths: Vec<FastqDuplicateStageTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct FastqDuplicateStageTruth {
    stage_id: String,
    cases: Vec<FastqDuplicateCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct FastqDuplicateCaseTruth {
    sample_id: String,
    tool_id: String,
    planned_tool_id: Option<String>,
    layout: String,
    reads_in: u64,
    duplicate_reads: u64,
    duplicate_fraction: f64,
    unique_reads: Option<u64>,
    retained_reads: Option<u64>,
    inspected_pair_count: Option<u64>,
    duplicate_status: Option<String>,
    outputs: Vec<FastqDuplicateOutputTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FastqDuplicateOutputTruth {
    read_end: String,
    records: Vec<FastqRecordTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FastqRecordTruth {
    read_id: String,
    sequence: String,
    quality: String,
}

#[derive(Debug, Deserialize)]
struct DetectDuplicatesSmokeReport {
    stage_id: String,
    case_count: usize,
    cases: Vec<DetectDuplicatesSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct DetectDuplicatesSmokeCase {
    sample_id: String,
    layout: String,
    reads_in: u64,
    duplicate_signal_reads: u64,
    duplicate_signal_fraction: f64,
    inspected_read_pair_count: u64,
    duplicate_status: String,
    report_json: String,
}

#[derive(Debug, Deserialize)]
struct RemoveDuplicatesSmokeReport {
    stage_id: String,
    sample_id: String,
    planned_tool_id: String,
    report_tool_id: String,
    paired_mode: String,
    input_reads: u64,
    duplicate_reads: u64,
    unique_reads: u64,
    output_reads: u64,
    dedup_fastq_gz: String,
    dedup_fastq_r2_gz: String,
    case_report_json: String,
}

#[derive(Debug, Deserialize)]
struct RemoveDuplicatesCaseReport {
    tool_id: String,
    reads_in: u64,
    reads_out: u64,
    duplicates_removed: u64,
    dedup_rate: f64,
    duplicate_classes: Vec<DuplicateClassRow>,
}

#[derive(Debug, Deserialize)]
struct DuplicateClassRow {
    class: String,
    reads_removed: u64,
}

pub(crate) fn validate_fastq_duplicates_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<FastqDuplicatesTruthValidationReport> {
    let manifest = load_fastq_duplicates_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;
    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "FASTQ duplicates truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "FASTQ duplicates truth bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_fastq_duplicates_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest.stage_ids)?;
    let expected_stage_map = expected
        .stage_truths
        .iter()
        .map(|stage| (stage.stage_id.as_str(), stage))
        .collect::<std::collections::BTreeMap<_, _>>();
    let actual_stage_map = actual
        .stage_truths
        .iter()
        .map(|stage| (stage.stage_id.as_str(), stage))
        .collect::<std::collections::BTreeMap<_, _>>();

    if expected_stage_map.len() != actual_stage_map.len() {
        return Err(anyhow!(
            "FASTQ duplicates truth stage count drifted: expected {}, observed {}",
            expected_stage_map.len(),
            actual_stage_map.len()
        ));
    }

    for stage_id in &manifest.stage_ids {
        let expected_stage = expected_stage_map
            .get(stage_id.as_str())
            .ok_or_else(|| anyhow!("expected bundle is missing stage `{stage_id}`"))?;
        let actual_stage = actual_stage_map
            .get(stage_id.as_str())
            .ok_or_else(|| anyhow!("observed bundle is missing stage `{stage_id}`"))?;
        if expected_stage != actual_stage {
            return Err(anyhow!("FASTQ duplicates truth drifted for stage `{stage_id}`"));
        }
    }

    let checked_cases = actual
        .stage_truths
        .iter()
        .flat_map(|stage| {
            stage
                .cases
                .iter()
                .map(|case| format!("{}:{}:{}", stage.stage_id, case.sample_id, case.tool_id))
        })
        .collect::<Vec<_>>();
    let validated_case_count = checked_cases.len();

    Ok(FastqDuplicatesTruthValidationReport {
        schema_version: FASTQ_DUPLICATES_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_stage_count: manifest.stage_ids.len(),
        validated_case_count,
        valid: true,
        checked_cases,
    })
}

fn load_fastq_duplicates_truth_manifest_path(
    manifest_path: &Path,
) -> Result<FastqDuplicatesTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &FastqDuplicatesTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != FASTQ_DUPLICATES_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "FASTQ duplicates truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            FASTQ_DUPLICATES_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != FASTQ_DUPLICATES_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "FASTQ duplicates truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            FASTQ_DUPLICATES_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "FASTQ duplicates truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "FASTQ duplicates truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "FASTQ duplicates truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    let required_stage_ids =
        REQUIRED_STAGE_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    if manifest.stage_ids != required_stage_ids {
        return Err(anyhow!(
            "FASTQ duplicates truth manifest `{}` must declare stage_ids {:?}",
            manifest_path.display(),
            required_stage_ids
        ));
    }
    Ok(())
}

fn load_fastq_duplicates_truth_bundle(expected_path: &Path) -> Result<FastqDuplicatesTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &FastqDuplicatesTruthManifest,
    bundle: &FastqDuplicatesTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != FASTQ_DUPLICATES_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "FASTQ duplicates truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            FASTQ_DUPLICATES_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "FASTQ duplicates truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let expected_stage_ids = manifest.stage_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let actual_stage_ids =
        bundle.stage_truths.iter().map(|stage| stage.stage_id.as_str()).collect::<Vec<_>>();
    if actual_stage_ids != expected_stage_ids {
        return Err(anyhow!(
            "FASTQ duplicates truth bundle `{}` must contain stages {:?}",
            expected_path.display(),
            expected_stage_ids
        ));
    }
    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    stage_ids: &[String],
) -> Result<FastqDuplicatesTruthBundle> {
    let mut stage_truths = Vec::with_capacity(stage_ids.len());
    for stage_id in stage_ids {
        let report_path = materialize_local_stage(repo_root, stage_id)
            .with_context(|| format!("materialize {stage_id}"))?;
        let stage_truth = match stage_id.as_str() {
            "fastq.detect_duplicates_premerge" => {
                build_detect_duplicates_stage_truth(repo_root, &report_path)?
            }
            "fastq.remove_duplicates" => {
                build_remove_duplicates_stage_truth(repo_root, &report_path)?
            }
            other => return Err(anyhow!("unsupported FASTQ duplicates truth stage `{other}`")),
        };
        stage_truths.push(stage_truth);
    }
    Ok(FastqDuplicatesTruthBundle {
        schema_version: FASTQ_DUPLICATES_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: FASTQ_DUPLICATES_TRUTH_FIXTURE_ID.to_string(),
        stage_truths,
    })
}

fn build_detect_duplicates_stage_truth(
    repo_root: &Path,
    report_path: &Path,
) -> Result<FastqDuplicateStageTruth> {
    let report: DetectDuplicatesSmokeReport = load_json(report_path)?;
    if report.stage_id != "fastq.detect_duplicates_premerge" {
        return Err(anyhow!(
            "detect-duplicates smoke report `{}` drifted to stage `{}`",
            report_path.display(),
            report.stage_id
        ));
    }
    if report.case_count != report.cases.len() {
        return Err(anyhow!(
            "detect-duplicates smoke report `{}` declared {} cases but stored {}",
            report_path.display(),
            report.case_count,
            report.cases.len()
        ));
    }

    let mut cases = Vec::with_capacity(report.cases.len());
    for case in report.cases {
        let case_report: serde_json::Value =
            load_json(repo_root.join(&case.report_json).as_path())?;
        let tool_id = case_report
            .get("tool_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("duplicate detection report is missing tool_id"))?
            .to_string();
        cases.push(FastqDuplicateCaseTruth {
            sample_id: case.sample_id,
            tool_id,
            planned_tool_id: None,
            layout: case.layout,
            reads_in: case.reads_in,
            duplicate_reads: case.duplicate_signal_reads,
            duplicate_fraction: case.duplicate_signal_fraction,
            unique_reads: None,
            retained_reads: None,
            inspected_pair_count: Some(case.inspected_read_pair_count),
            duplicate_status: Some(case.duplicate_status),
            outputs: Vec::new(),
        });
    }
    cases.sort_by(|left, right| {
        left.sample_id.cmp(&right.sample_id).then(left.tool_id.cmp(&right.tool_id))
    });
    Ok(FastqDuplicateStageTruth { stage_id: report.stage_id, cases })
}

fn build_remove_duplicates_stage_truth(
    repo_root: &Path,
    report_path: &Path,
) -> Result<FastqDuplicateStageTruth> {
    let report: RemoveDuplicatesSmokeReport = load_json(report_path)?;
    if report.stage_id != "fastq.remove_duplicates" {
        return Err(anyhow!(
            "remove-duplicates smoke report `{}` drifted to stage `{}`",
            report_path.display(),
            report.stage_id
        ));
    }
    let case_report: RemoveDuplicatesCaseReport =
        load_json(repo_root.join(&report.case_report_json).as_path())?;
    let dedup_r1 = repo_root.join(&report.dedup_fastq_gz);
    let dedup_r2 = repo_root.join(&report.dedup_fastq_r2_gz);
    let outputs = vec![
        FastqDuplicateOutputTruth {
            read_end: "r1".to_string(),
            records: read_fastq_records(&dedup_r1)?,
        },
        FastqDuplicateOutputTruth {
            read_end: "r2".to_string(),
            records: read_fastq_records(&dedup_r2)?,
        },
    ];
    let observed_retained_reads = outputs.iter().try_fold(0_u64, |sum, output| {
        u64::try_from(output.records.len())
            .context("record count fits in u64")
            .map(|count| sum + count)
    })?;

    let sample_id = report.sample_id.clone();
    let case = FastqDuplicateCaseTruth {
        sample_id,
        tool_id: report.report_tool_id,
        planned_tool_id: Some(report.planned_tool_id),
        layout: report.paired_mode,
        reads_in: report.input_reads,
        duplicate_reads: report.duplicate_reads,
        duplicate_fraction: case_report.dedup_rate,
        unique_reads: Some(report.unique_reads),
        retained_reads: Some(report.output_reads),
        inspected_pair_count: None,
        duplicate_status: None,
        outputs,
    };
    if case_report.tool_id != case.tool_id {
        return Err(anyhow!(
            "remove-duplicates case report `{}` drifted to tool_id `{}`",
            repo_root.join(&report.case_report_json).display(),
            case_report.tool_id
        ));
    }
    if case_report.reads_in != report.input_reads
        || case_report.reads_out != report.output_reads
        || case_report.duplicates_removed != report.duplicate_reads
    {
        return Err(anyhow!(
            "remove-duplicates summary/report counts drifted for `{}`",
            report.sample_id
        ));
    }
    let duplicate_class_reads_removed =
        case_report.duplicate_classes.iter().try_fold(0_u64, |total, row| {
            if row.class.trim().is_empty() {
                return Err(anyhow!(
                    "remove-duplicates duplicate_classes contain an empty class for `{}`",
                    report.sample_id
                ));
            }
            total.checked_add(row.reads_removed).ok_or_else(|| {
                anyhow!("duplicate class counts overflowed for `{}`", report.sample_id)
            })
        })?;
    if duplicate_class_reads_removed != report.duplicate_reads {
        return Err(anyhow!(
            "remove-duplicates duplicate_classes removed {} reads but summary reported {} for `{}`",
            duplicate_class_reads_removed,
            report.duplicate_reads,
            report.sample_id
        ));
    }
    if observed_retained_reads != report.output_reads {
        return Err(anyhow!(
            "remove-duplicates retained FASTQ records drifted for `{}`: observed {}, expected {}",
            report.sample_id,
            observed_retained_reads,
            report.output_reads
        ));
    }
    Ok(FastqDuplicateStageTruth { stage_id: report.stage_id, cases: vec![case] })
}

fn read_fastq_records(path: &Path) -> Result<Vec<FastqRecordTruth>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut reader: Box<dyn BufRead> = if path.to_string_lossy().ends_with(".gz") {
        Box::new(BufReader::new(MultiGzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };

    let mut records = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let read_id = line.trim_end().to_string();
        let mut sequence = String::new();
        if reader.read_line(&mut sequence)? == 0 {
            return Err(anyhow!("sequence line missing in {}", path.display()));
        }
        let mut plus_line = String::new();
        if reader.read_line(&mut plus_line)? == 0 {
            return Err(anyhow!("plus line missing in {}", path.display()));
        }
        if !read_id.starts_with('@') {
            return Err(anyhow!(
                "FASTQ record in {} is missing @ header: {}",
                path.display(),
                read_id
            ));
        }
        if !plus_line.starts_with('+') {
            return Err(anyhow!(
                "FASTQ record in {} is missing + separator for {}",
                path.display(),
                read_id
            ));
        }
        let mut quality = String::new();
        if reader.read_line(&mut quality)? == 0 {
            return Err(anyhow!("quality line missing in {}", path.display()));
        }
        records.push(FastqRecordTruth {
            read_id,
            sequence: sequence.trim_end().to_string(),
            quality: quality.trim_end().to_string(),
        });
    }
    Ok(records)
}

fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root.join(path)
    }
}

fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
