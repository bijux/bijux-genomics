//! Owner: bijux-dna-bench
//! Validation and no-clobber publication of completed benchmark attempts.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_execution_spec, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkEvidenceReference, BenchmarkExecutionSpec,
};
use serde::Deserialize;

use super::{
    BenchmarkExecutionAdmission, BenchmarkExecutionAdmissionPublication, BenchmarkMetricAssessment,
    BenchmarkOutputFileEvidence, BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1,
};

const MAX_EVIDENCE_BYTES: usize = 16 * 1024 * 1024;
const COMPLETION_SCHEMA_V1: &str = "bijux.runner.benchmark_completion.v1";
const VALIDATION_FILES: &[&str] = &[
    "validation/help.stderr",
    "validation/help.stdout",
    "validation/negative.stderr",
    "validation/negative.stdout",
    "validation/version.stderr",
    "validation/version.stdout",
    "validation/version.txt",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BenchmarkCompletion {
    schema_version: String,
    execution_spec_sha256: String,
    output_manifest: String,
}

/// Validate completed output closure and publish one immutable acceptance report.
///
/// # Errors
/// Returns an error when source evidence, output closure, hashes, metric predicates, downstream
/// probe, or an existing admission publication is missing, malformed, or conflicting.
pub fn admit_benchmark_execution_outputs(
    study_root: &Path,
    spec: &BenchmarkExecutionSpec,
    admitted_at_utc: &str,
) -> Result<BenchmarkExecutionAdmissionPublication> {
    validate_execution_spec(spec).context("validate benchmark execution specification")?;
    require_text(admitted_at_utc, "benchmark admission timestamp")?;
    let study_root = canonical_directory(study_root, "STUDY root")?;
    let result_root =
        canonical_directory(Path::new(&spec.outputs.result_root), "benchmark result root")?;
    if !result_root.starts_with(&study_root) {
        bail!("benchmark result root is outside STUDY");
    }
    let persisted_spec_path = result_root.join("execution-spec.json");
    let persisted_spec: BenchmarkExecutionSpec = read_json(&persisted_spec_path)?;
    if persisted_spec != *spec {
        bail!("persisted benchmark execution specification conflicts with admission request");
    }
    let execution_spec_sha256 = hash_regular_file(&persisted_spec_path)?.sha256;
    let completion_path = result_root.join("scheduler/completed.json");
    let completion: BenchmarkCompletion = read_json(&completion_path)?;
    if completion.schema_version != COMPLETION_SCHEMA_V1
        || completion.execution_spec_sha256 != execution_spec_sha256
        || completion.output_manifest != "scheduler/output-sha256.tsv"
    {
        bail!("benchmark completion evidence conflicts with execution specification");
    }
    let completion_sha256 = hash_regular_file(&completion_path)?.sha256;
    let output_manifest_path = result_root.join(&completion.output_manifest);
    let manifest_bytes = read_regular_file(&output_manifest_path)?;
    let outputs = validate_output_manifest(&result_root, spec, &manifest_bytes)?;
    validate_owned_output_closure(&result_root, &spec.outputs.required_files)?;
    let output_manifest_sha256 = bijux_dna_infra::hash_bytes_sha256(&manifest_bytes);
    let validation_files = VALIDATION_FILES
        .iter()
        .map(|relative| file_evidence(&result_root, relative))
        .collect::<Result<Vec<_>>>()?;
    if validation_files
        .iter()
        .find(|file| file.path == "validation/version.txt")
        .is_none_or(|file| file.size_bytes == 0)
    {
        bail!("benchmark version validation evidence is empty");
    }
    let downstream_probe = file_evidence(&result_root, &spec.interface.downstream_probe.artifact)?;
    let metrics = spec
        .acceptance
        .metrics
        .iter()
        .map(|metric| assess_metric(&result_root, metric))
        .collect::<Result<Vec<_>>>()?;
    if metrics.iter().any(|metric| !metric.passed) {
        bail!("benchmark execution fails one or more declared acceptance metrics");
    }
    let report = BenchmarkExecutionAdmission {
        schema_version: BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1.to_string(),
        identity: spec.identity.clone(),
        admitted_at_utc: admitted_at_utc.to_string(),
        execution_spec_sha256,
        completion_sha256,
        output_manifest_sha256,
        downstream_probe,
        validation_files,
        outputs,
        metrics,
    };
    let admission_root = result_root.join("admission");
    ensure_regular_directory(&admission_root)?;
    let report_path = admission_root.join("acceptance.json");
    ensure_json_identity(&report_path, &report)?;
    let report_file = hash_regular_file(&report_path)?;
    let relative = report_path
        .strip_prefix(&study_root)
        .context("benchmark admission report is outside STUDY")?;
    Ok(BenchmarkExecutionAdmissionPublication {
        report,
        evidence: BenchmarkEvidenceReference {
            uri: format!("study://{}", relative.to_string_lossy().replace('\\', "/")),
            sha256: report_file.sha256,
            size_bytes: report_file.size_bytes,
        },
    })
}

fn validate_output_manifest(
    result_root: &Path,
    spec: &BenchmarkExecutionSpec,
    bytes: &[u8],
) -> Result<Vec<BenchmarkOutputFileEvidence>> {
    let text = std::str::from_utf8(bytes).context("benchmark output manifest is not UTF-8")?;
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() != spec.outputs.required_files.len() {
        bail!("benchmark output manifest does not close over required outputs");
    }
    let mut outputs = Vec::new();
    for (line, expected_path) in lines.iter().zip(&spec.outputs.required_files) {
        let (path, digest) =
            line.split_once('\t').context("benchmark output manifest row is malformed")?;
        if path != expected_path || !is_digest(digest) {
            bail!("benchmark output manifest order or identity conflicts with specification");
        }
        let evidence = file_evidence(result_root, path)?;
        if evidence.sha256 != digest {
            bail!("benchmark required output changed after completion");
        }
        outputs.push(evidence);
    }
    Ok(outputs)
}

fn validate_owned_output_closure(result_root: &Path, required: &[String]) -> Result<()> {
    let required = required.iter().cloned().collect::<BTreeSet<_>>();
    let roots = required
        .iter()
        .filter_map(|path| Path::new(path).components().next())
        .map(|component| component.as_os_str().to_owned())
        .collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    for root in roots {
        for entry in walkdir::WalkDir::new(result_root.join(root)).follow_links(false) {
            let entry = entry.context("walk benchmark output closure")?;
            if entry.file_type().is_symlink() {
                bail!("benchmark output closure contains a symlink");
            }
            if entry.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(result_root)
                    .context("derive benchmark output relative path")?
                    .to_string_lossy()
                    .replace('\\', "/");
                observed.insert(relative);
            }
        }
    }
    if observed != required {
        bail!("benchmark output directory contains undeclared or missing files");
    }
    Ok(())
}

fn assess_metric(
    result_root: &Path,
    metric: &BenchmarkAcceptanceMetric,
) -> Result<BenchmarkMetricAssessment> {
    let value: serde_json::Value = read_json(&result_root.join(&metric.path))?;
    let observed_value = value
        .pointer(&metric.json_pointer)
        .with_context(|| format!("benchmark metric {} JSON pointer is absent", metric.id))?;
    let observed = typed_value(observed_value)?;
    let passed = compare_values(&observed, metric.operator, &metric.expected)?;
    Ok(BenchmarkMetricAssessment {
        id: metric.id.clone(),
        path: metric.path.clone(),
        json_pointer: metric.json_pointer.clone(),
        operator: metric.operator,
        expected: metric.expected.clone(),
        observed,
        unit: metric.unit.clone(),
        passed,
    })
}

fn typed_value(value: &serde_json::Value) -> Result<BenchmarkAcceptanceExpectedValue> {
    match value {
        serde_json::Value::Bool(value) => Ok(BenchmarkAcceptanceExpectedValue::Boolean(*value)),
        serde_json::Value::Number(value) => value.as_i64().map_or_else(
            || {
                value.as_f64().map(BenchmarkAcceptanceExpectedValue::Number).context(
                    "benchmark metric number cannot be represented as a finite floating value",
                )
            },
            |value| Ok(BenchmarkAcceptanceExpectedValue::Integer(value)),
        ),
        serde_json::Value::String(value) => {
            Ok(BenchmarkAcceptanceExpectedValue::Text(value.clone()))
        }
        _ => bail!("benchmark metric value is not scalar"),
    }
}

fn compare_values(
    observed: &BenchmarkAcceptanceExpectedValue,
    operator: BenchmarkAcceptanceOperator,
    expected: &BenchmarkAcceptanceExpectedValue,
) -> Result<bool> {
    match (numeric_value(observed), numeric_value(expected)) {
        (Some(observed), Some(expected)) => Ok(match operator {
            BenchmarkAcceptanceOperator::Equal => observed == expected,
            BenchmarkAcceptanceOperator::GreaterOrEqual => observed >= expected,
            BenchmarkAcceptanceOperator::GreaterThan => observed > expected,
            BenchmarkAcceptanceOperator::LessOrEqual => observed <= expected,
            BenchmarkAcceptanceOperator::LessThan => observed < expected,
        }),
        (None, None) if operator == BenchmarkAcceptanceOperator::Equal => Ok(observed == expected),
        _ => bail!("benchmark metric operator and value types are incompatible"),
    }
}

fn numeric_value(value: &BenchmarkAcceptanceExpectedValue) -> Option<f64> {
    match value {
        BenchmarkAcceptanceExpectedValue::Integer(value) => Some(*value as f64),
        BenchmarkAcceptanceExpectedValue::Number(value) if value.is_finite() => Some(*value),
        _ => None,
    }
}

fn file_evidence(root: &Path, relative: &str) -> Result<BenchmarkOutputFileEvidence> {
    let path = root.join(relative);
    let mut evidence = hash_regular_file(&path)?;
    evidence.path = relative.to_string();
    Ok(evidence)
}

fn hash_regular_file(path: &Path) -> Result<BenchmarkOutputFileEvidence> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect benchmark evidence {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("benchmark evidence is not a regular non-symlink file: {}", path.display());
    }
    Ok(BenchmarkOutputFileEvidence {
        path: path.display().to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(path)
            .with_context(|| format!("hash benchmark evidence {}", path.display()))?,
        size_bytes: metadata.len(),
    })
}

fn read_regular_file(path: &Path) -> Result<Vec<u8>> {
    let _ = hash_regular_file(path)?;
    bijux_dna_infra::read_to_end_bounded(path, MAX_EVIDENCE_BYTES)
        .with_context(|| format!("read benchmark evidence {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(&read_regular_file(path)?)
        .with_context(|| format!("parse benchmark evidence {}", path.display()))
}

fn ensure_regular_directory(path: &Path) -> Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!("benchmark admission root is not a regular directory");
            }
            Ok(())
        }
        Err(error) => Err(error).context("create benchmark admission root"),
    }
}

fn ensure_json_identity<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark admission report is not a regular file");
            }
            let existing: T = read_json(path)?;
            if existing != *value {
                bail!("benchmark admission report conflicts with immutable evidence");
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, value)
                .context("publish benchmark admission report")
        }
        Err(error) => Err(error).context("inspect benchmark admission report"),
    }
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a regular directory");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    Ok(canonical)
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_text(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() || value != value.trim() {
        bail!("{label} is empty or malformed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{
        BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator,
        BenchmarkAttemptIdentity, BenchmarkExecutionAcceptance, BenchmarkExecutionCachePolicy,
        BenchmarkExecutionCommands, BenchmarkExecutionDeterminism,
        BenchmarkExecutionDownstreamProbe, BenchmarkExecutionImage, BenchmarkExecutionInput,
        BenchmarkExecutionInputKind, BenchmarkExecutionInterface, BenchmarkExecutionMeasurement,
        BenchmarkExecutionOutputs, BenchmarkExecutionParameterEvidence,
        BenchmarkExecutionResources, BenchmarkExecutionRetryPolicy,
        BenchmarkExecutionRuntimePolicy, BenchmarkExecutionSourceLock, BenchmarkExecutionSpec,
        BenchmarkRunIdentity, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
    };

    use super::{admit_benchmark_execution_outputs, VALIDATION_FILES};

    #[test]
    fn admission_is_restart_safe_and_rejects_output_mutation() -> anyhow::Result<()> {
        let fixture = fixture()?;
        let first = admit_benchmark_execution_outputs(
            &fixture.study_root,
            &fixture.spec,
            "2026-08-05T04:00:00Z",
        )?;
        let recovered = admit_benchmark_execution_outputs(
            &fixture.study_root,
            &fixture.spec,
            "2026-08-05T04:00:00Z",
        )?;
        assert_eq!(first, recovered);
        assert_eq!(first.report.metrics.len(), 1);
        assert!(first.report.metrics[0].passed);

        std::fs::write(fixture.result_root.join("repetitions/repetition-1.metrics.json"), b"{}")?;
        assert!(admit_benchmark_execution_outputs(
            &fixture.study_root,
            &fixture.spec,
            "2026-08-05T04:00:00Z"
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn admission_rejects_undeclared_output_closure() -> anyhow::Result<()> {
        let fixture = fixture()?;
        std::fs::write(fixture.result_root.join("repetitions/undeclared.json"), b"{}")?;
        assert!(admit_benchmark_execution_outputs(
            &fixture.study_root,
            &fixture.spec,
            "2026-08-05T04:00:00Z"
        )
        .is_err_and(|error| error.to_string().contains("undeclared or missing")));
        Ok(())
    }

    struct Fixture {
        _root: tempfile::TempDir,
        study_root: std::path::PathBuf,
        result_root: std::path::PathBuf,
        spec: BenchmarkExecutionSpec,
    }

    fn fixture() -> anyhow::Result<Fixture> {
        let root = tempfile::tempdir()?;
        let study_root = root.path().canonicalize()?;
        let result_root = study_root.join(
            "results/benchmarks/run-2042-fastq-profile-read-lengths/seqkit/attempts/\
             human-short-read-length-comparison/attempt-001",
        );
        std::fs::create_dir_all(result_root.join("repetitions"))?;
        std::fs::create_dir_all(result_root.join("scheduler"))?;
        std::fs::create_dir_all(result_root.join("validation"))?;
        let required_files = vec![
            "repetitions/repetition-1.metrics.json".to_string(),
            "repetitions/repetition-1.report-qc.json".to_string(),
        ];
        std::fs::write(result_root.join(&required_files[0]), br#"{"read_count":42}"#)?;
        std::fs::write(result_root.join(&required_files[1]), br#"{"schema_version":"probe.v1"}"#)?;
        for relative in VALIDATION_FILES {
            std::fs::write(result_root.join(relative), b"evidence\n")?;
        }
        let spec = specification(&result_root, required_files.clone());
        let spec_path = result_root.join("execution-spec.json");
        std::fs::write(&spec_path, serde_json::to_vec_pretty(&spec)?)?;
        let spec_sha256 = bijux_dna_infra::hash_file_sha256(&spec_path)?;
        let mut manifest = String::new();
        for relative in &required_files {
            let digest = bijux_dna_infra::hash_file_sha256(&result_root.join(relative))?;
            manifest.push_str(&format!("{relative}\t{digest}\n"));
        }
        std::fs::write(result_root.join("scheduler/output-sha256.tsv"), manifest)?;
        std::fs::write(
            result_root.join("scheduler/completed.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": "bijux.runner.benchmark_completion.v1",
                "execution_spec_sha256": spec_sha256,
                "output_manifest": "scheduler/output-sha256.tsv"
            }))?,
        )?;
        Ok(Fixture { _root: root, study_root, result_root, spec })
    }

    fn specification(
        result_root: &std::path::Path,
        required_files: Vec<String>,
    ) -> BenchmarkExecutionSpec {
        BenchmarkExecutionSpec {
            schema_version: BENCHMARK_EXECUTION_SPEC_SCHEMA_V1.to_string(),
            identity: BenchmarkAttemptIdentity {
                campaign_id: "human-short-read-length-comparison".to_string(),
                run: BenchmarkRunIdentity {
                    run_id: "run-2042".to_string(),
                    domain: "fastq".to_string(),
                    stage_id: "fastq.profile_read_lengths".to_string(),
                },
                attempt_id: "attempt-001".to_string(),
                tool_id: "seqkit".to_string(),
            },
            operator: "operator".to_string(),
            prepared_at_utc: "2026-08-05T03:00:00Z".to_string(),
            source: BenchmarkExecutionSourceLock {
                repository_commit: "1".repeat(40),
                repository_path: "/study/worktrees/source".to_string(),
                contract_sha256: BTreeMap::from([
                    ("campaign_registry".to_string(), "2".repeat(64)),
                    ("container_catalog".to_string(), "3".repeat(64)),
                    ("image_inventory".to_string(), "4".repeat(64)),
                    ("stage_interface_registry".to_string(), "5".repeat(64)),
                    ("corpus_catalog".to_string(), "6".repeat(64)),
                    ("expectation_manifest".to_string(), "7".repeat(64)),
                ]),
            },
            image: BenchmarkExecutionImage {
                path: "/study/containers/seqkit.sif".to_string(),
                sha256: "8".repeat(64),
            },
            inputs: vec![BenchmarkExecutionInput {
                id: "fastq_err1".to_string(),
                kind: BenchmarkExecutionInputKind::Data,
                path: "/study/data/ERR1.fastq.gz".to_string(),
                mount_path: "/input/ERR1.fastq.gz".to_string(),
                sha256: "9".repeat(64),
            }],
            commands: BenchmarkExecutionCommands {
                version: vec!["seqkit".to_string(), "version".to_string()],
                help: vec!["seqkit".to_string(), "--help".to_string()],
                warmup: vec!["true".to_string()],
                measured: vec!["true".to_string()],
                negative: vec!["false".to_string()],
            },
            parameters: BenchmarkExecutionParameterEvidence {
                parameter_schema_version: "test.parameter.schema.v1".to_string(),
                defaults_provider: "bijux_dna_domain_fastq::read_length_profile_defaults"
                    .to_string(),
                overrides_path: "config-pipeline-parameters.yml".to_string(),
                overrides_sha256: "a".repeat(64),
                effective_path: "effective-parameters.json".to_string(),
                effective_sha256: "b".repeat(64),
            },
            interface: BenchmarkExecutionInterface {
                accepted_input_contracts: vec!["bijux.fastq.artifact.single_end.v1".to_string()],
                output_contract: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1".to_string(),
                normalized_result_contract: "benchmarks/contracts/schemas/result.json".to_string(),
                adapter: "bijux_dna_planner_fastq::profile_read_lengths".to_string(),
                downstream_probe: BenchmarkExecutionDownstreamProbe {
                    stage_id: "benchmark.fastq.profile_read_lengths.compare".to_string(),
                    input_contract: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1"
                        .to_string(),
                    source: "benchmarks/execution/fastq_read_lengths.awk".to_string(),
                    artifact: required_files[1].clone(),
                },
            },
            measurement: BenchmarkExecutionMeasurement {
                warmup_repetitions: 1,
                measured_repetitions: 3,
                resource_file_template: "repetitions/repetition-{repetition}.resource.tsv"
                    .to_string(),
                runtime_scope: "containerized_stage_end_to_end".to_string(),
            },
            outputs: BenchmarkExecutionOutputs {
                result_root: result_root.display().to_string(),
                mount_path: "/result".to_string(),
                required_files,
            },
            resources: BenchmarkExecutionResources {
                account: "lu2024-12-24".to_string(),
                partition: "compute".to_string(),
                cpus: 4,
                memory_mb: 8192,
                walltime: "01:00:00".to_string(),
                scratch_mb: 4096,
                retry: BenchmarkExecutionRetryPolicy {
                    max_attempts: 1,
                    backoff_seconds: 0,
                    retryable_exit_codes: Vec::new(),
                },
            },
            runtime: BenchmarkExecutionRuntimePolicy {
                locale: "C".to_string(),
                timezone: "UTC".to_string(),
                runtime_network: false,
                threads: 4,
                random_seed: None,
                cache_policy: BenchmarkExecutionCachePolicy::Warm,
                determinism: BenchmarkExecutionDeterminism::Deterministic,
            },
            acceptance: BenchmarkExecutionAcceptance {
                version_regex: "seqkit".to_string(),
                negative_exit_nonzero: true,
                metrics: vec![BenchmarkAcceptanceMetric {
                    id: "read_count".to_string(),
                    path: "repetitions/repetition-1.metrics.json".to_string(),
                    json_pointer: "/read_count".to_string(),
                    operator: BenchmarkAcceptanceOperator::GreaterThan,
                    expected: BenchmarkAcceptanceExpectedValue::Integer(0),
                    unit: "reads".to_string(),
                }],
            },
        }
    }
}
