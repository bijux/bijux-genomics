#![cfg_attr(test, allow(clippy::expect_used))]

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_stage_commands::materialize_local_stage;

pub(crate) const FASTQ_TRIMMING_TRUTH_FIXTURE_ID: &str = "fastq-trimming-truth";
pub(crate) const FASTQ_TRIMMING_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_trimming_truth.v1";
const FASTQ_TRIMMING_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_trimming_truth.expected.v1";
const FASTQ_TRIMMING_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_trimming_truth.validation.v1";
const REQUIRED_STAGE_IDS: &[&str] = &["fastq.trim_reads", "fastq.trim_polyg_tails"];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastqTrimmingTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    stage_ids: Vec<String>,
    source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqTrimmingTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_stage_count: usize,
    pub(crate) validated_case_count: usize,
    pub(crate) valid: bool,
    pub(crate) checked_cases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FastqTrimmingTruthBundle {
    schema_version: String,
    fixture_id: String,
    stage_truths: Vec<FastqTrimStageTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FastqTrimStageTruth {
    stage_id: String,
    cases: Vec<FastqTrimCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FastqTrimCaseTruth {
    sample_id: String,
    tool_id: String,
    layout: String,
    bases_removed: u64,
    quality_cutoff: Option<u64>,
    trimmed_tail_count: Option<u64>,
    outputs: Vec<FastqTrimOutputTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FastqTrimOutputTruth {
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
struct TrimReadsSmokeReport {
    stage_id: String,
    case_count: usize,
    all_cases_passed: bool,
    cases: Vec<TrimReadsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct TrimReadsSmokeCase {
    sample_id: String,
    tool_id: String,
    layout: String,
    bases_removed: u64,
    quality_cutoff: Option<u64>,
    trimmed_reads_r1: String,
    #[serde(default)]
    trimmed_reads_r2: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TrimPolygSmokeReport {
    stage_id: String,
    case_count: usize,
    cases: Vec<TrimPolygSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct TrimPolygSmokeCase {
    sample_id: String,
    tool_id: String,
    layout: String,
    bases_removed: u64,
    trimmed_tail_count: u64,
    trimmed_reads_r1: String,
    #[serde(default)]
    trimmed_reads_r2: Option<String>,
}

pub(crate) fn validate_fastq_trimming_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<FastqTrimmingTruthValidationReport> {
    let manifest = load_fastq_trimming_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(&manifest, manifest_path)?;
    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "FASTQ trimming truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("FASTQ trimming truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_fastq_trimming_truth_bundle(&expected_path)?;
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
            "FASTQ trimming truth stage count drifted: expected {}, observed {}",
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
            return Err(anyhow!("FASTQ trimming truth drifted for stage `{stage_id}`"));
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

    Ok(FastqTrimmingTruthValidationReport {
        schema_version: FASTQ_TRIMMING_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_stage_count: manifest.stage_ids.len(),
        validated_case_count,
        valid: true,
        checked_cases,
    })
}

fn load_fastq_trimming_truth_manifest_path(
    manifest_path: &Path,
) -> Result<FastqTrimmingTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    manifest: &FastqTrimmingTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != FASTQ_TRIMMING_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "FASTQ trimming truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            FASTQ_TRIMMING_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != FASTQ_TRIMMING_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "FASTQ trimming truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            FASTQ_TRIMMING_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "FASTQ trimming truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "FASTQ trimming truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    let required_stage_ids =
        REQUIRED_STAGE_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    if manifest.stage_ids != required_stage_ids {
        return Err(anyhow!(
            "FASTQ trimming truth manifest `{}` must declare stage_ids {:?}",
            manifest_path.display(),
            required_stage_ids
        ));
    }
    Ok(())
}

fn load_fastq_trimming_truth_bundle(expected_path: &Path) -> Result<FastqTrimmingTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &FastqTrimmingTruthManifest,
    bundle: &FastqTrimmingTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != FASTQ_TRIMMING_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "FASTQ trimming truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            FASTQ_TRIMMING_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "FASTQ trimming truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let expected_stage_ids = manifest.stage_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let actual_stage_ids =
        bundle.stage_truths.iter().map(|stage| stage.stage_id.as_str()).collect::<Vec<_>>();
    if actual_stage_ids != expected_stage_ids {
        return Err(anyhow!(
            "FASTQ trimming truth bundle `{}` must contain stages {:?}",
            expected_path.display(),
            expected_stage_ids
        ));
    }
    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    stage_ids: &[String],
) -> Result<FastqTrimmingTruthBundle> {
    let mut stage_truths = Vec::with_capacity(stage_ids.len());
    for stage_id in stage_ids {
        let report_path = materialize_local_stage(repo_root, stage_id)
            .with_context(|| format!("materialize {stage_id}"))?;
        let stage_truth = match stage_id.as_str() {
            "fastq.trim_reads" => build_trim_reads_stage_truth(repo_root, &report_path)?,
            "fastq.trim_polyg_tails" => build_trim_polyg_stage_truth(repo_root, &report_path)?,
            other => return Err(anyhow!("unsupported FASTQ trimming truth stage `{other}`")),
        };
        stage_truths.push(stage_truth);
    }
    Ok(FastqTrimmingTruthBundle {
        schema_version: FASTQ_TRIMMING_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: FASTQ_TRIMMING_TRUTH_FIXTURE_ID.to_string(),
        stage_truths,
    })
}

fn build_trim_reads_stage_truth(
    repo_root: &Path,
    report_path: &Path,
) -> Result<FastqTrimStageTruth> {
    let report: TrimReadsSmokeReport = load_json(report_path)?;
    if report.stage_id != "fastq.trim_reads" {
        return Err(anyhow!(
            "trim-reads smoke report `{}` drifted to stage `{}`",
            report_path.display(),
            report.stage_id
        ));
    }
    if !report.all_cases_passed {
        return Err(anyhow!(
            "trim-reads smoke report `{}` recorded failing cases",
            report_path.display()
        ));
    }
    if report.case_count != report.cases.len() {
        return Err(anyhow!(
            "trim-reads smoke report `{}` declared {} cases but stored {}",
            report_path.display(),
            report.case_count,
            report.cases.len()
        ));
    }

    let mut cases = report
        .cases
        .into_iter()
        .map(|case| build_trim_reads_case_truth(repo_root, case))
        .collect::<Result<Vec<_>>>()?;
    cases.sort_by(|left, right| {
        left.sample_id
            .cmp(&right.sample_id)
            .then(left.tool_id.cmp(&right.tool_id))
            .then(left.layout.cmp(&right.layout))
    });
    Ok(FastqTrimStageTruth { stage_id: report.stage_id, cases })
}

fn build_trim_reads_case_truth(
    repo_root: &Path,
    case: TrimReadsSmokeCase,
) -> Result<FastqTrimCaseTruth> {
    let trimmed_r1_path = repo_root.join(&case.trimmed_reads_r1);
    let mut outputs = vec![FastqTrimOutputTruth {
        read_end: "r1".to_string(),
        records: read_fastq_records(&trimmed_r1_path)?,
    }];
    if let Some(trimmed_reads_r2) = case.trimmed_reads_r2 {
        let path = repo_root.join(trimmed_reads_r2);
        outputs.push(FastqTrimOutputTruth {
            read_end: "r2".to_string(),
            records: read_fastq_records(&path)?,
        });
    }
    outputs.sort_by(|left, right| left.read_end.cmp(&right.read_end));
    Ok(FastqTrimCaseTruth {
        sample_id: case.sample_id,
        tool_id: case.tool_id,
        layout: case.layout,
        bases_removed: case.bases_removed,
        quality_cutoff: case.quality_cutoff,
        trimmed_tail_count: None,
        outputs,
    })
}

fn build_trim_polyg_stage_truth(
    repo_root: &Path,
    report_path: &Path,
) -> Result<FastqTrimStageTruth> {
    let report: TrimPolygSmokeReport = load_json(report_path)?;
    if report.stage_id != "fastq.trim_polyg_tails" {
        return Err(anyhow!(
            "trim-polyG smoke report `{}` drifted to stage `{}`",
            report_path.display(),
            report.stage_id
        ));
    }
    if report.case_count != report.cases.len() {
        return Err(anyhow!(
            "trim-polyG smoke report `{}` declared {} cases but stored {}",
            report_path.display(),
            report.case_count,
            report.cases.len()
        ));
    }

    let mut cases = report
        .cases
        .into_iter()
        .map(|case| build_trim_polyg_case_truth(repo_root, case))
        .collect::<Result<Vec<_>>>()?;
    cases.sort_by(|left, right| {
        left.sample_id
            .cmp(&right.sample_id)
            .then(left.tool_id.cmp(&right.tool_id))
            .then(left.layout.cmp(&right.layout))
    });
    Ok(FastqTrimStageTruth { stage_id: report.stage_id, cases })
}

fn build_trim_polyg_case_truth(
    repo_root: &Path,
    case: TrimPolygSmokeCase,
) -> Result<FastqTrimCaseTruth> {
    let trimmed_r1_path = repo_root.join(&case.trimmed_reads_r1);
    let mut outputs = vec![FastqTrimOutputTruth {
        read_end: "r1".to_string(),
        records: read_fastq_records(&trimmed_r1_path)?,
    }];
    if let Some(trimmed_reads_r2) = case.trimmed_reads_r2 {
        let path = repo_root.join(trimmed_reads_r2);
        outputs.push(FastqTrimOutputTruth {
            read_end: "r2".to_string(),
            records: read_fastq_records(&path)?,
        });
    }
    outputs.sort_by(|left, right| left.read_end.cmp(&right.read_end));
    Ok(FastqTrimCaseTruth {
        sample_id: case.sample_id,
        tool_id: case.tool_id,
        layout: case.layout,
        bases_removed: case.bases_removed,
        quality_cutoff: None,
        trimmed_tail_count: Some(case.trimmed_tail_count),
        outputs,
    })
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

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
