use super::*;
use anyhow::Result;
use bijux_dna_bench_model::{AllDomainOutputDeclarationStatus, AllDomainRenderedCommandStep};
use std::fs;

fn inputs(
) -> (AllDomainExpectedBenchmarkResultRow, AllDomainRenderedCommandRow, AllDomainOutputDeclarationRow)
{
    let expected = AllDomainExpectedBenchmarkResultRow {
        result_id: "bam:tiny:bam.sort:reference:sorter".into(),
        domain: "bam".into(),
        stage_id: "bam.sort".into(),
        tool_id: "sorter".into(),
        corpus_id: "tiny".into(),
        asset_profile_id: "reference".into(),
        expected_outputs: vec!["sorted_bam".into(), "qc_bundle".into()],
        expected_metrics: vec!["records".into()],
        report_section: "sorting".into(),
    };
    let command = AllDomainRenderedCommandRow {
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        readiness_kind: "stage".into(),
        benchmark_status: "benchmark_ready".into(),
        command_source: "fixture".into(),
        command_steps: vec![AllDomainRenderedCommandStep {
            step_id: "sort".into(),
            step_kind: "command".into(),
            consumes_previous_stdout: false,
            argv: vec!["must-not-execute".into()],
            command: "must-not-execute".into(),
        }],
        script_commands: vec!["must-not-execute".into()],
    };
    let output = AllDomainOutputDeclarationRow {
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        raw_outputs: expected.expected_outputs.clone(),
        normalized_metrics: vec!["sort_metrics".into()],
        logs: vec!["stdout".into()],
        manifest: "stage-result".into(),
        index_outputs: vec![],
        status: AllDomainOutputDeclarationStatus::Complete,
    };
    (expected, command, output)
}
const DESTINATION: &str = "artifacts/benchmark-smoke/simulated-runs/owner fixture";

#[test]
fn simulation_materializes_and_replays_explicitly_illustrative_outputs() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, command, outputs) = inputs();
    let run = || {
        fake_run_all_domain_benchmark_results(
            root.path(),
            Path::new(DESTINATION),
            std::slice::from_ref(&expected),
            std::slice::from_ref(&command),
            std::slice::from_ref(&outputs),
        )
    };
    let report = run()?;
    assert_eq!(report.result_count, 1);
    assert_eq!(report.created_output_count, 3);
    let row = &report.results[0];
    let stage: bijux_dna_bench_model::BenchmarkStageResultManifest =
        serde_json::from_slice(&fs::read(root.path().join(&row.stage_result_path))?)?;
    bijux_dna_bench_model::validate_benchmark_stage_result_manifest(&stage)?;
    assert_eq!(stage.runtime.mode, "benchmark_fake_run");
    assert_eq!(stage.command.rendered, "must-not-execute");
    assert_eq!(stage.runtime.elapsed_seconds, 1.25);
    let metrics_path = root.path().join(&row.metrics_path);
    let before: serde_json::Value = serde_json::from_slice(&fs::read(&metrics_path)?)?;
    let bam = row.outputs.iter().find(|output| output.artifact_id == "sorted_bam").unwrap();
    assert_eq!(fs::metadata(root.path().join(&bam.fake_run_path))?.len(), 0);
    let bundle = row.outputs.iter().find(|output| output.artifact_id == "qc_bundle").unwrap();
    let retained = root.path().join(&bundle.fake_run_path).join("unrelated-evidence");
    fs::write(&retained, b"keep this; it is not a simulation measurement")?;
    let second = run()?;
    assert_eq!(serde_json::to_value(&report)?, serde_json::to_value(second)?);
    assert_eq!(before, serde_json::from_slice::<serde_json::Value>(&fs::read(&metrics_path)?)?);
    assert_eq!(fs::read(retained)?, b"keep this; it is not a simulation measurement");
    let replay: AllDomainFakeRunsReport =
        serde_json::from_slice(&fs::read(root.path().join(&report.root_manifest_path))?)?;
    validate_all_domain_simulation_report(&replay)?;
    let mut wrong = replay.clone();
    wrong.schema_version = "unknown".into();
    assert!(validate_all_domain_simulation_report(&wrong).is_err());
    let mut wrong = replay.clone();
    wrong.results[0].declared_output_count += 1;
    wrong.results[0].created_output_count += 1;
    wrong.created_output_count += 1;
    assert!(validate_all_domain_simulation_report(&wrong).is_err());
    let mut wrong = replay.clone();
    wrong.domain_counts.clear();
    assert!(validate_all_domain_simulation_report(&wrong).is_err());
    let mut metrics = before;
    metrics["schema_version"] = "unknown".into();
    assert!(
        serde_json::from_value::<bijux_dna_bench_model::AllDomainFakeRunMetrics>(metrics).is_err()
    );
    let mut value = serde_json::to_value(replay)?;
    value["unexpected"] = true.into();
    assert!(serde_json::from_value::<AllDomainFakeRunsReport>(value).is_err());
    Ok(())
}

#[test]
fn simulation_rejects_ambiguous_or_unsafe_inputs_before_any_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, command, outputs) = inputs();
    let rejected = |e: &[AllDomainExpectedBenchmarkResultRow],
                    c: &[AllDomainRenderedCommandRow],
                    o: &[AllDomainOutputDeclarationRow]| {
        assert!(fake_run_all_domain_benchmark_results(
            root.path(),
            Path::new(DESTINATION),
            e,
            c,
            o
        )
        .is_err());
        assert!(!root.path().join("artifacts").exists());
    };
    rejected(
        &[expected.clone(), expected.clone()],
        std::slice::from_ref(&command),
        std::slice::from_ref(&outputs),
    );
    rejected(std::slice::from_ref(&expected), &[], std::slice::from_ref(&outputs));
    let mut wrong = command.clone();
    wrong.tool_id = "different-tool".into();
    rejected(std::slice::from_ref(&expected), &[wrong], std::slice::from_ref(&outputs));
    let mut wrong = outputs.clone();
    wrong.status = AllDomainOutputDeclarationStatus::Incomplete;
    rejected(std::slice::from_ref(&expected), std::slice::from_ref(&command), &[wrong]);
    let mut wrong = outputs.clone();
    wrong.raw_outputs = vec!["../escape_bam".into()];
    rejected(std::slice::from_ref(&expected), std::slice::from_ref(&command), &[wrong]);
    let mut extra_e = expected.clone();
    extra_e.result_id = "second".into();
    let mut extra_c = command.clone();
    extra_c.result_id = "second".into();
    let mut extra_o = outputs.clone();
    extra_o.result_id = "second".into();
    rejected(
        &[expected.clone(), extra_e],
        &[command.clone(), extra_c],
        &[outputs.clone(), extra_o],
    );
    assert!(fake_run_all_domain_benchmark_results(
        root.path(),
        Path::new("artifacts/benchmark-smoke/simulated-runs/../../escape"),
        &[expected],
        &[command],
        &[outputs]
    )
    .is_err());
    assert!(!root.path().join("artifacts").exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn simulation_rejects_existing_symlink_output_and_keeps_external_bytes() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, command, outputs) = inputs();
    let output = root
        .path()
        .join(DESTINATION)
        .join("bam/tiny/bam.sort/reference/sorter/declared-outputs/sorted.bam");
    fs::create_dir_all(output.parent().unwrap())?;
    let external = root.path().join("evidence.bam");
    fs::write(&external, b"retained")?;
    std::os::unix::fs::symlink(&external, &output)?;
    assert!(fake_run_all_domain_benchmark_results(
        root.path(),
        Path::new(DESTINATION),
        &[expected],
        &[command],
        &[outputs]
    )
    .is_err());
    assert_eq!(fs::read(external)?, b"retained");
    assert!(!root.path().join(DESTINATION).join("manifest.json").exists());
    Ok(())
}

#[path = "simulation/failure.rs"]
mod failure;

#[test]
fn missing_result_projection_binds_simulation_to_typed_manifest_observations() -> Result<()> {
    use bijux_dna_bench_model::{
        project_all_domain_missing_result_observation as project,
        AllDomainMissingResultObservation as Observation, AllDomainMissingResultStatus as Status,
        BenchmarkStageResultManifest,
    };
    let root = tempfile::tempdir()?;
    let (expected, command, outputs) = inputs();
    let simulated = fake_run_all_domain_benchmark_results(
        root.path(),
        Path::new(DESTINATION),
        std::slice::from_ref(&expected),
        std::slice::from_ref(&command),
        std::slice::from_ref(&outputs),
    )?;
    let source = &simulated.results[0];
    let manifest: BenchmarkStageResultManifest =
        serde_json::from_slice(&fs::read(root.path().join(&source.stage_result_path))?)?;
    let observe = |manifest: Option<&BenchmarkStageResultManifest>| {
        project(Observation { expected: &expected, outputs: &outputs, simulated: source, manifest })
    };
    let present = observe(Some(&manifest))?;
    assert_eq!(present.result_status, Status::Present);
    assert_eq!(present.observed_output_artifact_ids, present.expected_output_artifact_ids);
    let missing = observe(None)?;
    assert_eq!(missing.result_status, Status::MissingResult);
    assert_eq!(missing.expected_manifest_path, present.expected_manifest_path);
    assert!(missing.observed_output_artifact_ids.is_empty());
    let mut wrong = manifest.clone();
    wrong.tool.id = "foreign".into();
    assert!(observe(Some(&wrong)).is_err());
    let mut wrong = manifest;
    wrong.runtime.mode = "native_execution".into();
    assert!(observe(Some(&wrong)).is_err());
    let mut wrong = outputs.clone();
    wrong.asset_profile_id = "foreign".into();
    assert!(project(Observation {
        expected: &expected,
        outputs: &wrong,
        simulated: source,
        manifest: None
    })
    .is_err());
    Ok(())
}

#[path = "simulation/missing.rs"]
mod missing;
