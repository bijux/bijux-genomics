//! Owner: completion decisions from current repository observations and admitted declarations.
use crate::repo::simulation::completion::{self, CompletionPaths, ManifestObservation};
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::{
    all_domain_simulation_output_ids,
    completion::{
        AllDomainCompletionCheckRow, AllDomainCompletionRequiredFile, AllDomainCompletionStatus,
        REASON_EXECUTION_NOT_SUCCESSFUL, REASON_INVALID_MANIFEST, REASON_MANIFEST_OUTPUT_MISMATCH,
        REASON_MISSING_DECLARED_OUTPUTS, REASON_MISSING_MANIFEST,
        REASON_MISSING_NORMALIZED_METRICS, REASON_REQUIRED_FILES_INCOMPLETE,
    },
    AllDomainFakeRunResultReport, AllDomainOutputDeclarationRow, AllDomainSimulationInput,
    BenchmarkStageResultStatus,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect(
    paths: &CompletionPaths,
    inputs: &[AllDomainSimulationInput<'_>],
    simulated: &[AllDomainFakeRunResultReport],
) -> Result<Vec<AllDomainCompletionCheckRow>> {
    let by_id =
        simulated.iter().map(|row| (row.result_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    ensure!(
        by_id.len() == simulated.len() && inputs.len() == simulated.len(),
        "completion simulation result inventory differs from admitted sources"
    );
    let mut rows = inputs
        .iter()
        .map(|input| {
            let simulated = by_id
                .get(input.expected.result_id.as_str())
                .context("completion simulation result is absent")?;
            collect_row(paths, input, simulated)
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|a, b| {
        a.domain
            .cmp(&b.domain)
            .then_with(|| a.stage_id.cmp(&b.stage_id))
            .then_with(|| a.tool_id.cmp(&b.tool_id))
            .then_with(|| a.result_id.cmp(&b.result_id))
    });
    Ok(rows)
}

struct FileFacts {
    required: Vec<AllDomainCompletionRequiredFile>,
    missing_declared: Vec<String>,
    missing_normalized: Vec<String>,
}
struct ManifestFacts {
    exists: bool,
    valid: bool,
    outputs_match: bool,
    runtime_status: Option<String>,
    exit_code: Option<i32>,
}
impl ManifestFacts {
    fn successful(&self) -> bool {
        self.exit_code == Some(0) && self.runtime_status.as_deref() == Some("succeeded")
    }
}

fn collect_row(
    paths: &CompletionPaths,
    input: &AllDomainSimulationInput<'_>,
    simulated: &AllDomainFakeRunResultReport,
) -> Result<AllDomainCompletionCheckRow> {
    let files = observe_files(paths, input.outputs, simulated)?;
    let manifest = observe_manifest(paths, input, simulated)?;
    let failure_reasons = failure_reasons(&files, &manifest);
    let expected = input.expected;
    let declared_output_count = all_domain_simulation_output_ids(input.outputs).len();
    let ready_required_file_count =
        files.required.iter().filter(|file| file.exists && file.non_empty).count();
    Ok(AllDomainCompletionCheckRow {
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        command_script_path: simulated.command_script_path.clone(),
        stdout_path: simulated.stdout_path.clone(),
        stderr_path: simulated.stderr_path.clone(),
        metrics_path: simulated.metrics_path.clone(),
        stage_result_path: simulated.stage_result_path.clone(),
        manifest_exists: manifest.exists,
        manifest_valid: manifest.valid,
        manifest_output_match: manifest.outputs_match,
        exit_code_zero: manifest.exit_code == Some(0),
        runtime_status: manifest.runtime_status,
        exit_code: manifest.exit_code,
        declared_output_count,
        present_declared_output_count: declared_output_count - files.missing_declared.len(),
        missing_declared_output_ids: files.missing_declared,
        normalized_metrics_count: input.outputs.normalized_metrics.len(),
        present_normalized_metrics_count: input.outputs.normalized_metrics.len()
            - files.missing_normalized.len(),
        missing_normalized_metric_ids: files.missing_normalized,
        required_file_count: files.required.len(),
        ready_required_file_count,
        required_files: files.required,
        completion_status: if failure_reasons.is_empty() {
            AllDomainCompletionStatus::Complete
        } else {
            AllDomainCompletionStatus::Incomplete
        },
        failure_reasons,
    })
}

fn observe_files(
    paths: &CompletionPaths,
    outputs: &AllDomainOutputDeclarationRow,
    simulated: &AllDomainFakeRunResultReport,
) -> Result<FileFacts> {
    let required = [
        ("command_script", &simulated.command_script_path),
        ("stdout", &simulated.stdout_path),
        ("stderr", &simulated.stderr_path),
        ("metrics", &simulated.metrics_path),
    ]
    .into_iter()
    .map(|(id, path)| completion::required_file(paths, id, path))
    .collect::<Result<Vec<_>>>()?;
    let output_paths = simulated
        .outputs
        .iter()
        .map(|output| (output.artifact_id.as_str(), &output.fake_run_path))
        .collect::<BTreeMap<_, _>>();
    let mut missing_declared = Vec::new();
    let mut missing_normalized = Vec::new();
    let mut observed = BTreeMap::new();
    for id in all_domain_simulation_output_ids(outputs) {
        let path =
            output_paths.get(id.as_str()).context("completion simulation output is absent")?;
        let content = completion::output_content(paths, path, &id)?;
        if content.is_none() {
            missing_declared.push(id.clone());
        }
        observed.insert(id, content);
    }
    for id in &outputs.normalized_metrics {
        if observed.get(id).context("completion normalized output is undeclared")? != &Some(true) {
            missing_normalized.push(id.clone());
        }
    }
    Ok(FileFacts { required, missing_declared, missing_normalized })
}

fn observe_manifest(
    paths: &CompletionPaths,
    input: &AllDomainSimulationInput<'_>,
    simulated: &AllDomainFakeRunResultReport,
) -> Result<ManifestFacts> {
    let observed = completion::read_manifest(paths, &simulated.stage_result_path)?;
    let mut facts = ManifestFacts {
        exists: !matches!(observed, ManifestObservation::Missing),
        valid: false,
        outputs_match: false,
        runtime_status: None,
        exit_code: None,
    };
    if let ManifestObservation::Present(manifest) = observed {
        facts.valid = manifest.stage_id == input.expected.stage_id
            && manifest.tool.id == input.expected.tool_id;
        facts.runtime_status = Some(
            match manifest.runtime.status {
                BenchmarkStageResultStatus::Succeeded => "succeeded",
                BenchmarkStageResultStatus::Failed => "failed",
            }
            .to_owned(),
        );
        facts.exit_code = Some(manifest.runtime.exit_code);
        let expected =
            all_domain_simulation_output_ids(input.outputs).into_iter().collect::<BTreeSet<_>>();
        let actual = manifest
            .outputs
            .iter()
            .map(|output| output.artifact_id.clone())
            .collect::<BTreeSet<_>>();
        let output_paths = simulated
            .outputs
            .iter()
            .map(|output| (output.artifact_id.as_str(), &output.fake_run_path))
            .collect::<BTreeMap<_, _>>();
        facts.outputs_match = facts.valid
            && actual == expected
            && manifest.outputs.len() == expected.len()
            && manifest.outputs.iter().all(|output| {
                output.exists
                    && output_paths
                        .get(output.artifact_id.as_str())
                        .is_some_and(|path| **path == output.realized_path)
            });
    }
    Ok(facts)
}

fn failure_reasons(files: &FileFacts, manifest: &ManifestFacts) -> Vec<String> {
    let mut reasons = Vec::new();
    if !manifest.exists {
        reasons.push(REASON_MISSING_MANIFEST);
    } else if !manifest.valid {
        reasons.push(REASON_INVALID_MANIFEST);
    }
    if manifest.exists && manifest.valid && !manifest.successful() {
        reasons.push(REASON_EXECUTION_NOT_SUCCESSFUL);
    }
    if files.required.iter().any(|file| !file.exists || !file.non_empty) {
        reasons.push(REASON_REQUIRED_FILES_INCOMPLETE);
    }
    if !files.missing_declared.is_empty() {
        reasons.push(REASON_MISSING_DECLARED_OUTPUTS);
    }
    if !files.missing_normalized.is_empty() {
        reasons.push(REASON_MISSING_NORMALIZED_METRICS);
    }
    if manifest.exists && manifest.valid && !manifest.outputs_match {
        reasons.push(REASON_MANIFEST_OUTPUT_MISMATCH);
    }
    reasons.into_iter().map(str::to_owned).collect()
}
