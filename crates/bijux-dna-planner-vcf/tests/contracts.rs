use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_domain_vcf::contracts::PanelSelectionContext;
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use bijux_dna_planner_vcf::{
    explain_vcf_plan, plan_vcf_pipeline, plan_vcf_stage_plans, VcfPanelLock, VcfPipelineInputs,
};
use bijux_dna_runtime::stage_runner_contract::{
    ensure_stage_supported_by_runner, RunnerContractKind,
};

fn base_inputs(regime: CoverageRegime) -> VcfPipelineInputs {
    VcfPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        coverage_regime: regime,
        vcf: PathBuf::from("sample.vcf.gz"),
        out_dir: PathBuf::from("out"),
        stage_tool_overrides: BTreeMap::new(),
        requested_stages: None,
        panel_locks: vec![VcfPanelLock {
            panel_id: "1000g_phase3".to_string(),
            reference_build: "GRCh37".to_string(),
            panel_checksum_sha256: "a".repeat(64),
            index_checksum_sha256: "b".repeat(64),
            license_id: "CC-BY-4.0".to_string(),
        }],
        panel_selection: PanelSelectionContext {
            target_build: "GRCh37".to_string(),
            ancestry_hint: None,
            use_restricted_license: false,
        },
    }
}

fn assert_snapshot_json(name: &str, kind: &str, value: &serde_json::Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("{name}.{kind}.json"));
    let actual = serde_json::to_string_pretty(value).expect("serialize snapshot json");
    if std::env::var("UPDATE_SNAPSHOTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, format!("{actual}\n")).expect("write snapshot");
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read snapshot {}: {err}", path.display()));
    assert_eq!(actual, expected.trim_end(), "snapshot mismatch for {}", path.display());
}

fn snapshot_plan_and_explain(name: &str, inputs: &VcfPipelineInputs) {
    let plans = plan_vcf_stage_plans(inputs).unwrap_or_else(|err| panic!("stage plans: {err}"));
    let graph = plan_vcf_pipeline(inputs).unwrap_or_else(|err| panic!("pipeline graph: {err}"));
    let explain = explain_vcf_plan(inputs, &plans);

    let plan_json = serde_json::json!({
        "pipeline_id": graph.pipeline_id().to_string(),
        "planner_version": graph.planner_version().to_string(),
        "coverage_regime": inputs.coverage_regime,
        "stage_plans": plans,
        "edges": graph.edges(),
    });
    let explain_json = serde_json::to_value(explain).expect("serialize explain");

    assert_snapshot_json(name, "plan", &plan_json);
    assert_snapshot_json(name, "explain", &explain_json);
}

#[test]
fn vcf_downstream_snapshot_diploid_default() {
    let input = base_inputs(CoverageRegime::Diploid);
    snapshot_plan_and_explain("vcf_downstream_diploid_default", &input);
}

#[test]
fn vcf_downstream_snapshot_lowcov_default() {
    let input = base_inputs(CoverageRegime::LowCovGl);
    snapshot_plan_and_explain("vcf_downstream_lowcov_default", &input);
}

#[test]
fn vcf_downstream_snapshot_pseudohaploid_default() {
    let input = base_inputs(CoverageRegime::Pseudohaploid);
    snapshot_plan_and_explain("vcf_downstream_pseudohaploid_default", &input);
}

#[test]
fn vcf_downstream_snapshot_diploid_override_tools() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input
        .stage_tool_overrides
        .insert("vcf.phasing".to_string(), "eagle".to_string());
    input
        .stage_tool_overrides
        .insert("vcf.impute".to_string(), "minimac4".to_string());
    snapshot_plan_and_explain("vcf_downstream_diploid_override_tools", &input);
}

#[test]
fn vcf_downstream_snapshot_requested_subset_with_panel() {
    let mut input = base_inputs(CoverageRegime::LowCovGl);
    input.requested_stages = Some(vec![
        "vcf.prepare_reference_panel".to_string(),
        "vcf.call_gl".to_string(),
        "vcf.gl_propagation".to_string(),
        "vcf.impute".to_string(),
        "vcf.ibd".to_string(),
        "vcf.demography".to_string(),
        "vcf.stats".to_string(),
    ]);
    snapshot_plan_and_explain("vcf_downstream_subset_with_panel", &input);
}

#[test]
fn planner_stages_have_runtime_runner_coverage() {
    let graph =
        plan_vcf_pipeline(&base_inputs(CoverageRegime::Diploid)).expect("build vcf execution plan");
    for step in graph.steps() {
        ensure_stage_supported_by_runner(RunnerContractKind::Docker, step.stage_id.as_str())
            .unwrap_or_else(|err| panic!("missing runner for {}: {err}", step.stage_id.as_str()));
    }
}
