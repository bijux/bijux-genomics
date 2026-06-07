#![allow(clippy::expect_used, clippy::question_mark, clippy::unreadable_literal)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{
    build_plan_manifest, planner_refusal_from_message, ArtifactRole, ParameterResolutionTraceV1,
    PlanManifestBuildInputV1, PlanPolicy, PlannerParameterSourceV1, PlannerWarningCodeV1,
    PlannerWarningRecordV1, WorkflowInputArtifactV1, WorkflowManifestV1, WorkflowStageRequestV1,
};
use bijux_dna_domain_vcf::contracts::{
    ContigSpec, EntryVcfInvariantState, PanelMapInvariantState, PanelSelectionContext,
    SpeciesContext, VCF_COHORT_VALIDATION_CONTRACT, VCF_REPORT_COVERAGE_CONTRACT,
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
        call_bam: Some(PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam",
        )),
        call_bam_index: Some(PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam.bai",
        )),
        reference_fasta: Some(PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta",
        )),
        reference_panel_vcf: Some(PathBuf::from(
            "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_reference_panel.vcf",
        )),
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

#[test]
fn vcf_planner_renders_executable_bcftools_stage_templates_for_retained_rows() {
    for regime in [CoverageRegime::Diploid, CoverageRegime::LowCovGl, CoverageRegime::Pseudohaploid]
    {
        let plans = plan_vcf_stage_plans(&base_inputs(regime))
            .unwrap_or_else(|err| panic!("stage plans for {regime:?}: {err}"));
        for plan in plans.iter().filter(|plan| plan.tool_id.to_string() == "bcftools") {
            assert!(
                !plan.command.template.is_empty(),
                "bcftools stage {} must keep a real command template",
                plan.stage_id
            );
            assert!(
                !plan.command.template.iter().any(|part| part == "--help"),
                "bcftools stage {} must not fall back to --help placeholder rendering: {:?}",
                plan.stage_id,
                plan.command.template
            );
        }
    }
}

#[test]
fn vcf_planner_renders_executable_angsd_stage_templates_for_low_coverage_rows() {
    for regime in [CoverageRegime::LowCovGl, CoverageRegime::Pseudohaploid] {
        let plans = plan_vcf_stage_plans(&base_inputs(regime))
            .unwrap_or_else(|err| panic!("stage plans for {regime:?}: {err}"));
        for plan in plans.iter().filter(|plan| plan.tool_id.to_string() == "angsd") {
            assert!(
                !plan.command.template.is_empty(),
                "angsd stage {} must keep a real command template",
                plan.stage_id
            );
            assert_eq!(
                plan.command.template.first().map(String::as_str),
                Some("angsd"),
                "angsd stage {} must start with the real angsd binary",
                plan.stage_id
            );
            assert!(
                !plan.command.template.iter().any(|part| part == "--help"),
                "angsd stage {} must not fall back to --help placeholder rendering: {:?}",
                plan.stage_id,
                plan.command.template
            );
        }
    }
}

#[test]
fn vcf_planner_renders_executable_plink2_stage_templates_for_cohort_rows() {
    for regime in [CoverageRegime::Diploid, CoverageRegime::LowCovGl, CoverageRegime::Pseudohaploid]
    {
        let plans = plan_vcf_stage_plans(&base_inputs(regime))
            .unwrap_or_else(|err| panic!("stage plans for {regime:?}: {err}"));
        let plink2_rows =
            plans.iter().filter(|plan| plan.tool_id.to_string() == "plink2").collect::<Vec<_>>();
        assert!(!plink2_rows.is_empty(), "expected plink2 cohort-analysis rows for {regime:?}");
        for plan in plink2_rows {
            assert!(
                !plan.command.template.is_empty(),
                "plink2 stage {} must keep a real command template",
                plan.stage_id
            );
            let joined = plan.command.template.join(" ");
            assert!(
                joined.contains("plink2"),
                "plink2 stage {} must keep the concrete plink2 binary in its command template: {:?}",
                plan.stage_id,
                plan.command.template
            );
            assert!(
                !plan.command.template.iter().any(|part| part == "--help"),
                "plink2 stage {} must not fall back to --help placeholder rendering: {:?}",
                plan.stage_id,
                plan.command.template
            );
        }
    }
}

#[test]
fn vcf_planner_renders_executable_plink_stage_templates_when_overridden() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.requested_stages = Some(vec!["vcf.qc".to_string(), "vcf.admixture".to_string()]);
    input.stage_tool_overrides.insert("vcf.qc".to_string(), "plink".to_string());
    input.stage_tool_overrides.insert("vcf.admixture".to_string(), "plink".to_string());

    let plans =
        plan_vcf_stage_plans(&input).unwrap_or_else(|err| panic!("stage plans with plink: {err}"));
    let plink_rows = plans
        .iter()
        .filter(|plan| {
            plan.tool_id.to_string() == "plink"
                && matches!(plan.stage_id.as_str(), "vcf.qc" | "vcf.admixture")
        })
        .collect::<Vec<_>>();
    assert_eq!(plink_rows.len(), 2, "expected governed plink overrides for qc and admixture");
    for plan in plink_rows {
        assert!(
            !plan.command.template.is_empty(),
            "plink stage {} must keep a real command template",
            plan.stage_id
        );
        assert_eq!(
            plan.command.template.first().map(String::as_str),
            Some("plink"),
            "plink stage {} must start with the real plink binary",
            plan.stage_id
        );
        assert!(
            !plan.command.template.iter().any(|part| part == "--help"),
            "plink stage {} must not fall back to --help placeholder rendering: {:?}",
            plan.stage_id,
            plan.command.template
        );
    }
}

#[test]
fn vcf_planner_renders_executable_eigensoft_stage_templates_when_overridden() {
    for stage_id in ["vcf.pca", "vcf.population_structure"] {
        let mut input = base_inputs(CoverageRegime::Diploid);
        input.requested_stages = Some(vec![stage_id.to_string()]);
        input.stage_tool_overrides.insert(stage_id.to_string(), "eigensoft".to_string());

        let plans = plan_vcf_stage_plans(&input)
            .unwrap_or_else(|err| panic!("stage plans with eigensoft for {stage_id}: {err}"));
        let plan = plans
            .iter()
            .find(|plan| {
                plan.tool_id.to_string() == "eigensoft" && plan.stage_id.as_str() == stage_id
            })
            .unwrap_or_else(|| panic!("expected governed eigensoft override for {stage_id}"));
        assert!(
            !plan.command.template.is_empty(),
            "eigensoft stage {} must keep a real command template",
            plan.stage_id
        );
        let joined = plan.command.template.join(" ");
        assert!(
            joined.contains("convertf"),
            "eigensoft stage {} must render convertf input conversion: {:?}",
            plan.stage_id,
            plan.command.template
        );
        assert!(
            joined.contains("smartpca"),
            "eigensoft stage {} must render smartpca execution: {:?}",
            plan.stage_id,
            plan.command.template
        );
        assert!(
            !plan.command.template.iter().any(|part| part == "--help"),
            "eigensoft stage {} must not fall back to --help placeholder rendering: {:?}",
            plan.stage_id,
            plan.command.template
        );
    }
}

#[test]
fn vcf_planner_renders_executable_phasing_stage_templates_for_retained_backends() {
    let cases = [
        (CoverageRegime::Diploid, "shapeit5"),
        (CoverageRegime::Diploid, "eagle"),
        (CoverageRegime::Diploid, "beagle"),
    ];

    for (regime, tool_id) in cases {
        let mut input = base_inputs(regime);
        input.requested_stages = Some(vec!["vcf.phasing".to_string()]);
        input.stage_tool_overrides.insert("vcf.phasing".to_string(), tool_id.to_string());

        let plans = plan_vcf_stage_plans(&input)
            .unwrap_or_else(|err| panic!("stage plans for phasing backend {tool_id}: {err}"));
        let phasing_plan = plans
            .iter()
            .find(|plan| plan.stage_id.to_string() == "vcf.phasing")
            .unwrap_or_else(|| panic!("phasing plan for {tool_id}"));
        let joined = phasing_plan.command.template.join(" ");

        assert!(
            !joined.contains("--help"),
            "phasing backend {tool_id} must not fall back to placeholder rendering: {:?}",
            phasing_plan.command.template
        );
        assert!(
            joined.contains("reference_panel_vcf")
                || joined.contains("vcf_mini_reference_panel.vcf"),
            "phasing backend {tool_id} must keep panel input wiring: {:?}",
            phasing_plan.command.template
        );
        assert!(
            joined.contains("map.tsv.gz"),
            "phasing backend {tool_id} must keep genetic-map wiring: {:?}",
            phasing_plan.command.template
        );
        assert!(
            joined.contains("bcftools index"),
            "phasing backend {tool_id} must keep phased-vcf indexing: {:?}",
            phasing_plan.command.template
        );

        match tool_id {
            "shapeit5" => {
                assert!(
                    joined.contains("shapeit5 phase_common"),
                    "shapeit5 phasing must render phase_common: {:?}",
                    phasing_plan.command.template
                );
                assert!(joined.contains("--reference"));
                assert!(joined.contains("--map"));
            }
            "eagle" => {
                assert!(
                    joined.contains("eagle"),
                    "eagle phasing must render eagle command: {:?}",
                    phasing_plan.command.template
                );
                assert!(joined.contains("--vcfTarget"));
                assert!(joined.contains("--geneticMapFile"));
            }
            "beagle" => {
                assert!(
                    joined.contains("beagle"),
                    "beagle phasing must render beagle command: {:?}",
                    phasing_plan.command.template
                );
                assert!(joined.contains("gt='") || joined.contains("gt="));
                assert!(joined.contains("ref='") || joined.contains("ref="));
                assert!(joined.contains("map='") || joined.contains("map="));
            }
            other => panic!("unexpected phasing backend {other}"),
        }
    }
}

#[test]
fn vcf_planner_renders_executable_imputation_stage_templates_for_retained_backends() {
    let cases = [
        (CoverageRegime::Diploid, "vcf.imputation_metrics", "beagle"),
        (CoverageRegime::LowCovGl, "vcf.imputation_metrics", "glimpse"),
        (CoverageRegime::Diploid, "vcf.imputation_metrics", "impute5"),
        (CoverageRegime::Diploid, "vcf.imputation_metrics", "minimac4"),
        (CoverageRegime::Diploid, "vcf.impute", "beagle"),
        (CoverageRegime::LowCovGl, "vcf.impute", "glimpse"),
        (CoverageRegime::Diploid, "vcf.impute", "impute5"),
        (CoverageRegime::Diploid, "vcf.impute", "minimac4"),
    ];

    for (regime, stage_id, tool_id) in cases {
        let mut input = base_inputs(regime);
        input.requested_stages = Some(vec![stage_id.to_string()]);
        input.stage_tool_overrides.insert(stage_id.to_string(), tool_id.to_string());

        let plans = plan_vcf_stage_plans(&input).unwrap_or_else(|err| {
            panic!("stage plans for imputation backend {tool_id} on {stage_id}: {err}")
        });
        let stage_plan = plans
            .iter()
            .find(|plan| plan.stage_id.as_str() == stage_id)
            .unwrap_or_else(|| panic!("stage plan for {tool_id} on {stage_id}"));
        let joined = stage_plan.command.template.join(" ");

        assert!(
            !joined.contains("--help") && !joined.contains(" impute --input vcf"),
            "imputation backend {tool_id} on {stage_id} must not fall back to placeholder rendering: {:?}",
            stage_plan.command.template
        );
        assert!(
            joined.contains("vcf_mini_reference_panel.vcf")
                || joined.contains("reference_panel_vcf")
                || (stage_id == "vcf.impute" && joined.contains("panel.m3vcf.gz")),
            "imputation backend {tool_id} on {stage_id} must keep panel input wiring: {:?}",
            stage_plan.command.template
        );
        if tool_id != "minimac4" || stage_id == "vcf.imputation_metrics" {
            assert!(
                joined.contains("recombination_map.tsv.gz"),
                "imputation backend {tool_id} on {stage_id} must keep map wiring: {:?}",
                stage_plan.command.template
            );
        }

        if stage_id == "vcf.imputation_metrics" {
            assert!(
                joined.contains("printf")
                    && joined.contains("bijux.vcf.imputation_metrics.v1")
                    && joined.contains("\"stage_id\":\"vcf.imputation_metrics\"")
                    && joined.contains(&format!("\"tool_id\":\"{tool_id}\""))
                    && joined.contains("imputation_metrics.json"),
                "imputation metrics stage for {tool_id} must render a governed report command: {:?}",
                stage_plan.command.template
            );
            continue;
        }

        assert!(
            joined.contains("bcftools index"),
            "imputation backend {tool_id} on {stage_id} must keep indexed VCF output wiring: {:?}",
            stage_plan.command.template
        );

        match tool_id {
            "beagle" => {
                assert!(joined.contains("beagle"));
                assert!(joined.contains("gt='") || joined.contains("gt="));
                assert!(joined.contains("ref='") || joined.contains("ref="));
                assert!(joined.contains("map='") || joined.contains("map="));
                assert!(joined.contains("impute=true"));
            }
            "glimpse" => {
                assert!(joined.contains("GLIMPSE_phase"));
                assert!(joined.contains("--reference"));
                assert!(joined.contains("--input-region"));
                assert!(joined.contains("--output-region"));
            }
            "impute5" => {
                assert!(joined.contains("impute5"));
                assert!(joined.contains("--g"));
                assert!(joined.contains("--h"));
                assert!(joined.contains("--m"));
                assert!(joined.contains("--r"));
            }
            "minimac4" => {
                assert!(joined.contains("minimac4"));
                assert!(joined.contains("panel.m3vcf.gz"));
                assert!(joined.contains("--refHaps"));
                assert!(joined.contains("--prefix"));
            }
            other => panic!("unexpected imputation backend {other}"),
        }
    }
}

#[test]
fn vcf_planner_renders_executable_descent_stage_templates_for_retained_backends() {
    let cases = [
        (CoverageRegime::Diploid, "vcf.roh", "plink2"),
        (CoverageRegime::Diploid, "vcf.ibd", "germline"),
        (CoverageRegime::Diploid, "vcf.ibd", "ibdseq"),
        (CoverageRegime::Diploid, "vcf.ibd", "ibdhap"),
        (CoverageRegime::Diploid, "vcf.demography", "ibdne"),
    ];

    for (regime, stage_id, tool_id) in cases {
        let mut input = base_inputs(regime);
        input.requested_stages = Some(if stage_id == "vcf.demography" {
            vec!["vcf.ibd".to_string(), stage_id.to_string()]
        } else {
            vec![stage_id.to_string()]
        });
        input.stage_tool_overrides.insert(stage_id.to_string(), tool_id.to_string());

        let plans = plan_vcf_stage_plans(&input).unwrap_or_else(|err| {
            panic!("stage plans for descent backend {tool_id} on {stage_id}: {err}")
        });
        let stage_plan = plans
            .iter()
            .find(|plan| plan.stage_id.as_str() == stage_id)
            .unwrap_or_else(|| panic!("stage plan for {tool_id} on {stage_id}"));
        let joined = stage_plan.command.template.join(" ");

        assert!(
            !joined.contains("--help"),
            "descent backend {tool_id} on {stage_id} must not fall back to placeholder rendering: {:?}",
            stage_plan.command.template
        );

        match (stage_id, tool_id) {
            ("vcf.roh", "plink2") => {
                assert!(joined.contains("plink2"));
                assert!(joined.contains("--homozyg"));
                assert!(joined.contains("--vcf"));
            }
            ("vcf.ibd", "germline") => {
                assert!(joined.contains("germline"));
                assert!(joined.contains("plink2 --vcf"));
                assert!(joined.contains("-make-bed") || joined.contains("--make-bed"));
                assert!(joined.contains("-output"));
            }
            ("vcf.ibd", "ibdseq") => {
                assert!(joined.contains("ibdseq"));
                assert!(joined.contains("--vcf"));
                assert!(joined.contains(".segments.tsv"));
            }
            ("vcf.ibd", "ibdhap") => {
                assert!(joined.contains("ibdhap"));
                assert!(joined.contains("--vcf"));
                assert!(joined.contains(".segments.tsv"));
            }
            ("vcf.demography", "ibdne") => {
                assert!(joined.contains("ibdne"));
                assert!(joined.contains("ibd_segments.tsv"));
                assert!(joined.contains("--out"));
            }
            other => panic!("unexpected descent backend {:?}", other),
        }
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

fn vcf_workflow_manifest(advisory_stats: bool) -> WorkflowManifestV1 {
    let mut manifest = WorkflowManifestV1::new("vcf", "vcf-downstream");
    manifest.inputs = vec![WorkflowInputArtifactV1 {
        artifact_id: "vcf".to_string(),
        role: ArtifactRole::Variant,
        path: PathBuf::from("sample.vcf.gz"),
        layout: None,
        compression: None,
        format_id: Some("vcf.gz".to_string()),
    }];
    manifest.requested_stages = vec![
        WorkflowStageRequestV1 { stage_id: "vcf.filter".to_string(), advisory_only: false },
        WorkflowStageRequestV1 { stage_id: "vcf.stats".to_string(), advisory_only: advisory_stats },
    ];
    manifest
}

fn vcf_plan_manifest_value(
    inputs: &VcfPipelineInputs,
    advisory_stats: bool,
) -> anyhow::Result<serde_json::Value> {
    let stage_plans = plan_vcf_stage_plans(inputs)?;
    let graph = plan_vcf_pipeline(inputs)?;
    let effective_parameters_by_step = stage_plans
        .iter()
        .map(|plan| {
            (
                plan.stage_instance_id
                    .as_ref()
                    .map_or_else(|| plan.stage_id.to_string(), std::string::ToString::to_string),
                plan.effective_params.clone(),
            )
        })
        .collect();
    let parameter_traces = stage_plans
        .iter()
        .filter_map(|plan| {
            let serde_json::Value::Object(map) = &plan.effective_params else {
                return None;
            };
            let Some((name, value)) = map.iter().next() else {
                return None;
            };
            Some(ParameterResolutionTraceV1 {
                step_id: plan
                    .stage_instance_id
                    .as_ref()
                    .map_or_else(|| plan.stage_id.to_string(), std::string::ToString::to_string),
                stage_id: plan.stage_id.to_string(),
                parameter: name.clone(),
                source: PlannerParameterSourceV1::PlannerInferred,
                resolved_value: value.clone(),
                detail: "derived from stage effective params".to_string(),
            })
        })
        .collect();
    let warnings = if advisory_stats {
        vec![PlannerWarningRecordV1 {
            code: PlannerWarningCodeV1::AdvisoryStage,
            stage_id: Some("vcf.stats".to_string()),
            message: "stats stage treated as advisory in this contract".to_string(),
        }]
    } else {
        Vec::new()
    };
    let manifest = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: vcf_workflow_manifest(advisory_stats),
        graph,
        stage_contract_refs: Vec::new(),
        effective_parameters_by_step,
        parameter_traces,
        refusal_records: Vec::new(),
        warning_records: warnings,
    })?;
    Ok(serde_json::to_value(manifest)?)
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
    assert!(!explain.phasing_imputation_boundary_contracts.is_empty());
    assert!(!explain.population_guardrail_contracts.is_empty());
    assert!(!explain.cohort_analysis_boundary_contracts.is_empty());
    assert_eq!(
        explain.cohort_validation_contract.schema_version,
        VCF_COHORT_VALIDATION_CONTRACT.schema_version
    );
    assert_eq!(
        explain.report_coverage_contract.schema_version,
        VCF_REPORT_COVERAGE_CONTRACT.schema_version
    );
    assert!(explain.stages.iter().any(|stage| {
        stage.stage_id == "vcf.call_diploid"
            && !stage.artifact_classes.is_empty()
            && stage.calling_mode_contract.is_some()
    }));
}

#[test]
fn vcf_planner_refuses_imputation_without_explicit_panel_identity() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.requested_stages = Some(vec![
        "vcf.call_diploid".to_string(),
        "vcf.phasing".to_string(),
        "vcf.impute".to_string(),
        "vcf.stats".to_string(),
    ]);
    input.panel_locks.clear();
    input.panel_id = None;

    let err = plan_vcf_stage_plans(&input)
        .expect_err("imputation without explicit panel identity must fail");

    assert!(
        err.to_string().contains("explicit panel identity is required"),
        "unexpected planner refusal: {err}"
    );
}

#[test]
fn vcf_explain_surfaces_selected_panel_for_phasing_without_prepare_panel_stage() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.requested_stages = Some(vec![
        "vcf.call_diploid".to_string(),
        "vcf.phasing".to_string(),
        "vcf.impute".to_string(),
        "vcf.stats".to_string(),
    ]);
    let plans = plan_vcf_stage_plans(&input).unwrap_or_else(|err| panic!("stage plans: {err}"));
    let explain = explain_vcf_plan(&input, &plans);

    assert_eq!(
        explain.selected_panel.as_ref().map(|panel| panel.panel_id.as_str()),
        Some("1000g_phase3")
    );
    assert!(explain.panel_selection_reason.contains("panel selected"));
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
fn vcf_planner_keeps_bam_call_inputs_after_reference_panel_preparation() {
    let input = base_inputs(CoverageRegime::LowCovGl);
    let plans = plan_vcf_stage_plans(&input).unwrap_or_else(|err| panic!("stage plans: {err}"));
    let call_gl = plans
        .iter()
        .find(|plan| plan.stage_id.to_string() == "vcf.call_gl")
        .expect("call_gl stage plan");

    assert_eq!(
        call_gl.io.inputs[0].path,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam"
        )
    );
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
fn vcf_planner_aligns_ibd_and_demography_io_contracts() {
    let input = base_inputs(CoverageRegime::Diploid);
    let plans = plan_vcf_stage_plans(&input).unwrap_or_else(|err| panic!("stage plans: {err}"));
    let ibd =
        plans.iter().find(|plan| plan.stage_id.to_string() == "vcf.ibd").expect("ibd stage plan");
    let demography = plans
        .iter()
        .find(|plan| plan.stage_id.to_string() == "vcf.demography")
        .expect("demography stage plan");

    assert_eq!(ibd.io.outputs[0].path, PathBuf::from("out/ibd_segments.tsv"));
    assert_eq!(demography.io.inputs[0].path, PathBuf::from("out/ibd_segments.tsv"));
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

#[test]
fn vcf_happy_plan_manifest_snapshot_is_stable() {
    let input = base_inputs(CoverageRegime::Diploid);
    let value =
        vcf_plan_manifest_value(&input, false).unwrap_or_else(|err| panic!("plan manifest: {err}"));
    assert_snapshot_json("vcf_happy_plan_manifest", "manifest", &value);
}

#[test]
fn vcf_advisory_plan_manifest_snapshot_is_stable() {
    let input = base_inputs(CoverageRegime::Diploid);
    let value =
        vcf_plan_manifest_value(&input, true).unwrap_or_else(|err| panic!("plan manifest: {err}"));
    assert_snapshot_json("vcf_advisory_plan_manifest", "manifest", &value);
}

#[test]
fn vcf_refusal_manifest_snapshot_is_stable() {
    let mut input = base_inputs(CoverageRegime::Diploid);
    input.requested_stages = Some(vec!["vcf.stats".to_string(), "vcf.filter".to_string()]);
    let err = plan_vcf_stage_plans(&input).expect_err("out-of-order stages must fail");
    let refusal = planner_refusal_from_message(None, &err.to_string());
    let payload = serde_json::json!({
        "workflow_manifest": vcf_workflow_manifest(false),
        "refusal_records": [refusal],
    });
    assert_snapshot_json("vcf_refusal_manifest", "manifest", &payload);
}
