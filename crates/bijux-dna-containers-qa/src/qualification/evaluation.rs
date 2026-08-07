use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use regex::RegexBuilder;
use serde_json::Value;

use super::{
    ContainerQualificationAssessment, ContainerQualificationPlan, QualificationFailure,
    QualificationFailureClass, QualificationMetricEvidence, QualificationMetricPlan,
    QualificationOutputEvidence, QualificationProbeKind, QualificationProbeObservation,
};

/// Evaluate probe observations, declared outputs, and acceptance metrics as one QA verdict.
///
/// # Errors
/// Returns an error when observations are missing or duplicated, output roots are invalid, or an
/// evidence digest cannot be computed. Expected scientific rejection is returned as typed failure
/// evidence rather than an evaluation error.
pub fn evaluate_container_qualification(
    plan: &ContainerQualificationPlan,
    observations: &[QualificationProbeObservation],
    result_root: &Path,
    output_root: &Path,
) -> Result<ContainerQualificationAssessment> {
    if !result_root.is_dir() || !output_root.is_dir() || !output_root.starts_with(result_root) {
        bail!("qualification result and output roots are invalid");
    }
    let observations = indexed_observations(observations)?;
    let mut failures = assess_probes(plan, &observations)?;
    let outputs = inspect_required_outputs(plan, result_root, output_root, &mut failures)?;
    let metrics = evaluate_metrics(plan, output_root, &mut failures)?;
    Ok(ContainerQualificationAssessment { failures, outputs, metrics })
}

fn indexed_observations(
    observations: &[QualificationProbeObservation],
) -> Result<BTreeMap<QualificationProbeKind, &QualificationProbeObservation>> {
    let mut indexed = BTreeMap::new();
    for observation in observations {
        if indexed.insert(observation.kind, observation).is_some() {
            bail!("qualification probe observation is duplicated: {}", observation.kind.as_str());
        }
    }
    let expected = [
        QualificationProbeKind::Version,
        QualificationProbeKind::Help,
        QualificationProbeKind::Real,
        QualificationProbeKind::Negative,
    ];
    if expected.iter().any(|kind| !indexed.contains_key(kind)) || indexed.len() != expected.len() {
        bail!("qualification probe observations do not close over the required probes");
    }
    Ok(indexed)
}

fn assess_probes(
    plan: &ContainerQualificationPlan,
    observations: &BTreeMap<QualificationProbeKind, &QualificationProbeObservation>,
) -> Result<Vec<QualificationFailure>> {
    let mut failures = Vec::new();
    let version = observations[&QualificationProbeKind::Version];
    if version.exit_code != 0 {
        failures.push(failure(
            QualificationFailureClass::VersionExit,
            "version",
            "version command failed",
        ));
    }
    let version_text = format!("{}{}", version.stdout, version.stderr);
    if !RegexBuilder::new(&plan.version_regex)
        .case_insensitive(true)
        .build()?
        .is_match(version_text.trim_end())
    {
        failures.push(failure(
            QualificationFailureClass::VersionMismatch,
            "version",
            "version output did not match the catalog contract",
        ));
    }
    for (kind, class, detail) in [
        (QualificationProbeKind::Help, QualificationFailureClass::HelpExit, "help command failed"),
        (QualificationProbeKind::Real, QualificationFailureClass::RealExit, "real command failed"),
    ] {
        if observations[&kind].exit_code != 0 {
            failures.push(failure(class, kind.as_str(), detail));
        }
    }
    if observations[&QualificationProbeKind::Negative].exit_code == 0 {
        failures.push(failure(
            QualificationFailureClass::NegativeExit,
            "negative",
            "negative command unexpectedly succeeded",
        ));
    }
    Ok(failures)
}

fn inspect_required_outputs(
    plan: &ContainerQualificationPlan,
    result_root: &Path,
    output_root: &Path,
    failures: &mut Vec<QualificationFailure>,
) -> Result<Vec<QualificationOutputEvidence>> {
    let mut outputs = Vec::new();
    for relative in &plan.required_files {
        let output = output_root.join(relative);
        match fs::symlink_metadata(&output) {
            Ok(metadata)
                if !metadata.file_type().is_symlink()
                    && metadata.is_file()
                    && metadata.len() > 0 =>
            {
                outputs.push(QualificationOutputEvidence {
                    path: output.strip_prefix(result_root)?.to_string_lossy().into_owned(),
                    size_bytes: metadata.len(),
                    sha256: bijux_dna_infra::hash_file_sha256(&output)?,
                });
            }
            _ => failures.push(failure(
                QualificationFailureClass::RequiredOutput,
                &relative.display().to_string(),
                &format!("required output is absent or empty: {}", relative.display()),
            )),
        }
    }
    Ok(outputs)
}

fn evaluate_metrics(
    plan: &ContainerQualificationPlan,
    output_root: &Path,
    failures: &mut Vec<QualificationFailure>,
) -> Result<Vec<QualificationMetricEvidence>> {
    plan.metrics
        .iter()
        .map(|metric| {
            let evaluation = evaluate_metric(output_root, metric);
            let (observed, passed) = match evaluation {
                Ok((observed, true)) => (observed, true),
                Ok((observed, false)) => {
                    failures.push(failure(
                        QualificationFailureClass::MetricRejected,
                        &metric.id,
                        &format!(
                            "metric {} rejected {} {} {}",
                            metric.id, observed, metric.operator, metric.expected
                        ),
                    ));
                    (observed, false)
                }
                Err(error) => {
                    failures.push(failure(
                        QualificationFailureClass::MetricUnreadable,
                        &metric.id,
                        &format!("metric {} cannot be evaluated: {error:#}", metric.id),
                    ));
                    (Value::Null, false)
                }
            };
            Ok(QualificationMetricEvidence {
                id: metric.id.clone(),
                path: metric.path.clone(),
                json_pointer: metric.json_pointer.clone(),
                operator: metric.operator.clone(),
                expected: metric.expected.clone(),
                observed,
                unit: metric.unit.clone(),
                status: if passed { "passed" } else { "failed" }.to_string(),
            })
        })
        .collect()
}

fn evaluate_metric(output_root: &Path, metric: &QualificationMetricPlan) -> Result<(Value, bool)> {
    let source = output_root.join(&metric.path);
    let metadata = fs::symlink_metadata(&source)
        .with_context(|| format!("inspect metric source {}", source.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("metric source is not a regular file: {}", source.display());
    }
    let payload: Value =
        serde_json::from_slice(&bijux_dna_infra::read_to_end_bounded(&source, 16 * 1024 * 1024)?)?;
    let observed = payload
        .pointer(&metric.json_pointer)
        .with_context(|| format!("metric JSON pointer does not resolve: {}", metric.id))?
        .clone();
    Ok((observed.clone(), compare_metric(&observed, &metric.operator, &metric.expected)?))
}

fn compare_metric(observed: &Value, operator: &str, expected: &Value) -> Result<bool> {
    if operator == "eq" {
        return Ok(observed == expected);
    }
    let observed = observed.as_f64().context("ordered metric observation is not numeric")?;
    let expected = expected.as_f64().context("ordered metric expectation is not numeric")?;
    match operator {
        "ge" => Ok(observed >= expected),
        "gt" => Ok(observed > expected),
        "le" => Ok(observed <= expected),
        "lt" => Ok(observed < expected),
        _ => bail!("metric operator is unsupported: {operator}"),
    }
}

fn failure(class: QualificationFailureClass, subject: &str, detail: &str) -> QualificationFailure {
    QualificationFailure { class, subject: subject.to_string(), detail: detail.to_string() }
}
