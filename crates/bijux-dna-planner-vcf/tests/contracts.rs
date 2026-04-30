#![allow(clippy::expect_used, clippy::unreadable_literal)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_domain_vcf::contracts::{
    ContigSpec, EntryVcfInvariantState, PanelMapInvariantState, PanelSelectionContext,
    SpeciesContext,
};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use bijux_dna_planner_vcf::{
    explain_vcf_plan, plan_vcf_pipeline, plan_vcf_stage_plans, resolve_reference_context_report,
    ChunkPlanSettings, VcfPanelLock, VcfPipelineInputs,
};

fn base_inputs(regime: CoverageRegime) -> VcfPipelineInputs {
    VcfPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        coverage_regime: regime,
        mean_depth_x: None,
        vcf: PathBuf::from("sample.vcf.gz"),
        out_dir: PathBuf::from("out"),
        stage_tool_overrides: BTreeMap::new(),
        requested_stages: None,
        panel_locks: vec![VcfPanelLock {
            panel_id: "1000g_phase3".to_string(),
            reference_build: "GRCh38".to_string(),
            panel_checksum_sha256: "a".repeat(64),
            index_checksum_sha256: "b".repeat(64),
            license_id: "CC-BY-4.0".to_string(),
        }],
        panel_id: None,
        map_id: None,
        panel_selection: PanelSelectionContext {
            target_build: "GRCh38".to_string(),
            ancestry_hint: None,
            use_restricted_license: false,
        },
        species_context: SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![
                ContigSpec { name: "1".to_string(), length_bp: 248956422 },
                ContigSpec { name: "2".to_string(), length_bp: 242193529 },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: Some(regime),
        },
        entry_vcf_invariants: EntryVcfInvariantState {
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            sorted_by_contig_and_pos: true,
            bgzip_compressed: true,
            tabix_index_present: true,
            sample_ids_non_empty_unique: true,
            ploidy_constraints_ok: true,
        },
        panel_map_invariants: PanelMapInvariantState {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            phased_or_gl_compatible: true,
            format_requirements_ok: true,
            sample_count_ok: true,
            license_allowed: true,
            checksums_match: true,
        },
        pipeline_domain: "vcf".to_string(),
        chunking: ChunkPlanSettings::default(),
        stage_param_overrides: BTreeMap::new(),
    }
}

fn assert_snapshot_json(name: &str, kind: &str, value: &serde_json::Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("bijux-dna-planner-vcf__contracts__{name}.{kind}.json"));
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
    input.stage_tool_overrides.insert("vcf.phasing".to_string(), "eagle".to_string());
    input.stage_tool_overrides.insert("vcf.impute".to_string(), "minimac4".to_string());
    snapshot_plan_and_explain("vcf_downstream_diploid_override_tools", &input);
}

#[test]
fn vcf_downstream_snapshot_requested_subset_with_panel() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.requested_stages = Some(vec![
        "vcf.prepare_reference_panel".to_string(),
        "vcf.call_diploid".to_string(),
        "vcf.impute".to_string(),
        "vcf.ibd".to_string(),
        "vcf.demography".to_string(),
        "vcf.stats".to_string(),
    ]);
    snapshot_plan_and_explain("vcf_downstream_subset_with_panel", &input);
}

#[test]
fn vcf_planner_refuses_duplicate_requested_stages() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.requested_stages = Some(vec![
        "vcf.call_diploid".to_string(),
        "vcf.filter".to_string(),
        "vcf.filter".to_string(),
        "vcf.stats".to_string(),
    ]);

    let err = plan_vcf_stage_plans(&input).expect_err("duplicate requested stages must fail");

    assert!(
        err.to_string().contains("duplicate stage vcf.filter"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_requested_stage_outside_resolved_coverage() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.requested_stages = Some(vec!["vcf.call_gl".to_string(), "vcf.stats".to_string()]);

    let err = plan_vcf_stage_plans(&input).expect_err("coverage-incompatible stage must fail");

    assert!(
        err.to_string().contains("vcf.call_gl is not supported"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_requested_stages_out_of_downstream_order() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.requested_stages = Some(vec!["vcf.stats".to_string(), "vcf.filter".to_string()]);

    let err = plan_vcf_stage_plans(&input).expect_err("out-of-order stages must fail");

    assert!(
        err.to_string().contains("downstream ordering violation"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_edna_or_pollen_domain() {
    let mut input = base_inputs(CoverageRegime::LowCovGl);
    input.pipeline_domain = "edna".to_string();
    let err = plan_vcf_stage_plans(&input).expect_err("edna domain must be refused");
    assert!(err.to_string().contains("eDNA/pollen"), "unexpected refusal message: {err}");
}

#[test]
fn vcf_planner_refuses_pollen_domain_with_science_specific_message() {
    let mut input = base_inputs(CoverageRegime::LowCovGl);
    input.pipeline_domain = "ancient_pollen".to_string();

    let err = plan_vcf_stage_plans(&input).expect_err("pollen domain must be refused");

    assert!(err.to_string().contains("eDNA/pollen"), "unexpected refusal message: {err}");
}

#[test]
fn vcf_planner_refuses_unknown_stage_knob_override() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.stage_param_overrides.insert(
        "vcf.impute".to_string(),
        serde_json::json!({
            "definitely_unknown_knob": 1
        }),
    );
    let err = plan_vcf_stage_plans(&input).expect_err("unknown knob override must fail");
    assert!(
        err.to_string().contains("unknown knob for vcf.impute"),
        "unexpected validation error: {err}"
    );
}

#[test]
fn vcf_planner_refuses_param_override_for_unplanned_stage() {
    let mut input = base_inputs(CoverageRegime::Pseudohaploid);
    input.stage_param_overrides.insert(
        "vcf.impute".to_string(),
        serde_json::json!({
            "ne": 20000
        }),
    );

    let err = plan_vcf_stage_plans(&input).expect_err("unplanned param override must fail");

    assert!(
        err.to_string().contains("stage_param_overrides key vcf.impute"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_unknown_param_override_stage() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.stage_param_overrides.insert("vcf.not_real".to_string(), serde_json::json!({}));

    let err = plan_vcf_stage_plans(&input).expect_err("unknown param override stage must fail");

    assert!(
        err.to_string().contains("unknown stage_param_overrides key vcf.not_real"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_reference_context_report_surfaces_build_alias_and_checksum_contracts() {
    let report = resolve_reference_context_report(&base_inputs(CoverageRegime::Diploid))
        .unwrap_or_else(|err| panic!("resolve reference context report: {err}"));

    assert_eq!(report.schema_version, "bijux.vcf.reference_context_report.v1");
    assert_eq!(report.build_id, "GRCh38");
    assert!(!report.contig_naming_scheme.is_empty());
    assert_eq!(report.fasta_sha256.len(), 64);
    assert!(report.alias_count > 0);
    assert_eq!(report.panel_id, "hsapiens_grch38_mini");
    assert_eq!(report.map_id, "hsapiens_grch38_chr_map");
    assert!(report.vcf_index_required);
}

#[test]
fn vcf_explain_surfaces_artifact_classes_and_guardrails() {
    let plans = plan_vcf_stage_plans(&base_inputs(CoverageRegime::Diploid))
        .unwrap_or_else(|err| panic!("stage plans: {err}"));
    let explain = explain_vcf_plan(&base_inputs(CoverageRegime::Diploid), &plans);

    assert_eq!(explain.reference_context.schema_version, "bijux.vcf.reference_context_report.v1");
    assert!(!explain.panel_boundary_contracts.is_empty());
    assert!(!explain.population_guardrail_contracts.is_empty());
    assert!(explain.stages.iter().any(|stage| {
        stage.stage_id == "vcf.call_diploid"
            && !stage.artifact_classes.is_empty()
            && stage.calling_mode_contract.is_some()
    }));
}

#[test]
fn vcf_planner_refuses_tool_not_declared_in_required_tools_lists() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input
        .stage_tool_overrides
        .insert("vcf.impute".to_string(), "not_a_real_required_tool".to_string());
    let err = plan_vcf_stage_plans(&input).expect_err("unknown required tool override must fail");
    assert!(err.to_string().contains("required_tools_vcf"), "unexpected planner refusal: {err}");
}

#[test]
fn vcf_planner_refuses_tool_override_for_unplanned_stage() {
    let mut input = base_inputs(CoverageRegime::Pseudohaploid);
    input.stage_tool_overrides.insert("vcf.impute".to_string(), "beagle".to_string());

    let err = plan_vcf_stage_plans(&input).expect_err("unplanned tool override must fail");

    assert!(
        err.to_string().contains("stage_tool_overrides key vcf.impute"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_unknown_tool_override_stage() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.stage_tool_overrides.insert("vcf.not_real".to_string(), "bcftools".to_string());

    let err = plan_vcf_stage_plans(&input).expect_err("unknown tool override stage must fail");

    assert!(
        err.to_string().contains("unknown stage_tool_overrides key vcf.not_real"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_resolves_coverage_regime_from_mean_depth() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.mean_depth_x = Some(0.3);
    let plans = plan_vcf_stage_plans(&input).unwrap_or_else(|err| panic!("stage plans: {err}"));
    assert!(
        plans.iter().any(|p| p.stage_id.to_string() == "vcf.call_gl"),
        "low-depth run should resolve to lowcov_gl default stages"
    );
    assert!(
        !plans.iter().any(|p| p.stage_id.to_string() == "vcf.call_diploid"),
        "low-depth run must not keep diploid default stage list"
    );
    let explain = explain_vcf_plan(&input, &plans);
    assert_eq!(explain.resolved_coverage_regime, CoverageRegime::LowCovGl);
}

#[test]
fn vcf_planner_keeps_sample_vcf_after_reference_panel_preparation() {
    let input = base_inputs(CoverageRegime::LowCovGl);
    let plans = plan_vcf_stage_plans(&input).unwrap_or_else(|err| panic!("stage plans: {err}"));
    let call_gl = plans
        .iter()
        .find(|plan| plan.stage_id.to_string() == "vcf.call_gl")
        .expect("call_gl stage plan");

    assert_eq!(call_gl.io.inputs[0].path, PathBuf::from("sample.vcf.gz"));
}

#[test]
fn vcf_planner_keeps_variant_stream_after_report_stages() {
    let input = base_inputs(CoverageRegime::Diploid);
    let plans = plan_vcf_stage_plans(&input).unwrap_or_else(|err| panic!("stage plans: {err}"));
    let roh =
        plans.iter().find(|plan| plan.stage_id.to_string() == "vcf.roh").expect("roh stage plan");
    let stats = plans
        .iter()
        .find(|plan| plan.stage_id.to_string() == "vcf.stats")
        .expect("stats stage plan");

    assert_eq!(roh.io.inputs[0].path, PathBuf::from("out/postprocess_vcf.vcf.gz"));
    assert_eq!(stats.io.inputs[0].path, PathBuf::from("out/postprocess_vcf.vcf.gz"));
}

#[test]
fn vcf_planner_refuses_zero_parallel_chunk_setting() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.chunking.max_parallel_chunks = 0;

    let err = plan_vcf_stage_plans(&input).expect_err("zero parallel chunk setting must fail");

    assert!(
        err.to_string().contains("max_parallel_chunks must be > 0"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_conflicting_chunk_contig_filters() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.chunking.chr_include = vec!["1".to_string()];
    input.chunking.chr_exclude = vec!["1".to_string()];

    let err = plan_vcf_stage_plans(&input).expect_err("conflicting chunk filters must fail");

    assert!(
        err.to_string().contains("both chr_include and chr_exclude"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_unknown_chunk_include_contig() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.chunking.chr_include = vec!["chr01".to_string()];

    let err = plan_vcf_stage_plans(&input).expect_err("unknown included contig must fail");

    assert!(
        err.to_string().contains("chr_include contains unknown contig `chr01`"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_unknown_chunk_exclude_contig() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.chunking.chr_exclude = vec!["chrM".to_string()];

    let err = plan_vcf_stage_plans(&input).expect_err("unknown excluded contig must fail");

    assert!(
        err.to_string().contains("chr_exclude contains unknown contig `chrM`"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_chunk_filters_with_no_regions() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.chunking.chr_exclude = vec!["1".to_string(), "2".to_string()];

    let err = plan_vcf_stage_plans(&input).expect_err("empty chunk plan must fail");

    assert!(
        err.to_string().contains("produced no region chunks"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_planner_refuses_duplicate_chunk_filter_entries() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.chunking.chr_include = vec!["1".to_string(), "1".to_string()];

    let err = plan_vcf_stage_plans(&input).expect_err("duplicate chunk filter must fail");

    assert!(
        err.to_string().contains("chr_include contains duplicate contig `1`"),
        "unexpected planner refusal: {err}"
    );
}
