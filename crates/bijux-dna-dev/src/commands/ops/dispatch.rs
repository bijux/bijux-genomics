use super::{
    assets, docs, examples, hpc, lab, smoke, tooling, verification, NativeOpsCommandKey,
    OpsCommandOutcome, Result, Workspace,
};

pub(super) fn run_native_ops_command(
    key: NativeOpsCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    match &key {
        NativeOpsCommandKey::AssetsRefreshGolden => assets::assets_refresh_golden(workspace, args),
        NativeOpsCommandKey::AssetsRefreshReference => {
            assets::assets_refresh_reference(workspace, args)
        }
        NativeOpsCommandKey::AssetsRefreshToy => assets::assets_refresh_toy(workspace, args),
        NativeOpsCommandKey::AssetsValidateReference => {
            assets::assets_validate_reference(workspace, args)
        }
        NativeOpsCommandKey::DocsCheckDocAssets => docs::docs_check_doc_assets(workspace, args),
        NativeOpsCommandKey::DocsCheckDocDepth => docs::docs_check_doc_depth(workspace, args),
        NativeOpsCommandKey::DocsCheckDocLinks => docs::docs_check_doc_links(workspace, args),
        NativeOpsCommandKey::DocsCheckDocRootLayout => {
            docs::docs_check_doc_root_layout(workspace, args)
        }
        NativeOpsCommandKey::DocsCheckDocsGraph => docs::docs_check_docs_graph(workspace, args),
        NativeOpsCommandKey::DocsCheckDomainDocReferences => {
            docs::docs_check_domain_doc_references(workspace, args)
        }
        NativeOpsCommandKey::DocsCheckGeneratedDocs => {
            docs::docs_check_generated_docs(workspace, args)
        }
        NativeOpsCommandKey::DocsCheckNoPlaceholderLanguage => {
            docs::docs_check_no_placeholder_language(workspace, args)
        }
        NativeOpsCommandKey::DocsCheckRootPollution => {
            docs::docs_check_root_pollution(workspace, args)
        }
        NativeOpsCommandKey::DocsCheckDocMajorDepth => {
            docs::docs_check_doc_major_depth(workspace, args)
        }
        NativeOpsCommandKey::ExamplesGenerateIndex => {
            examples::examples_generate_index(workspace, args)
        }
        NativeOpsCommandKey::ExamplesCheckIndex => examples::examples_check_index(workspace, args),
        NativeOpsCommandKey::ExamplesRun => examples::examples_run(workspace, args),
        NativeOpsCommandKey::ExamplesCheckDrift => examples::examples_check_drift(workspace, args),
        NativeOpsCommandKey::HpcValidateFrontendConstraints => {
            hpc::hpc_validate_frontend_constraints(workspace, args)
        }
        NativeOpsCommandKey::HpcRunFrontendMiniE2e => {
            hpc::hpc_run_frontend_mini_e2e(workspace, args)
        }
        NativeOpsCommandKey::HpcBenchmarkSyncPull => hpc::hpc_benchmark_sync_pull(workspace, args),
        NativeOpsCommandKey::HpcBenchmarkSyncPush => hpc::hpc_benchmark_sync_push(workspace, args),
        NativeOpsCommandKey::LabRunBench => lab::lab_run_bench(workspace, args),
        NativeOpsCommandKey::LabRunPipelines => lab::lab_run_pipelines(workspace, args),
        NativeOpsCommandKey::SmokeRun => smoke::smoke_run(workspace, args),
        NativeOpsCommandKey::SmokeBam => smoke::smoke_bam(workspace, args),
        NativeOpsCommandKey::SmokeFastq => smoke::smoke_fastq(workspace, args),
        NativeOpsCommandKey::TestControlPlaneSmoke => {
            verification::test_control_plane_smoke(workspace, args)
        }
        NativeOpsCommandKey::TestTriage => verification::test_triage(workspace, args),
        NativeOpsCommandKey::TestReproduceFailure => {
            verification::test_reproduce_failure(workspace, args)
        }
        NativeOpsCommandKey::TestFastqGoldRepro => {
            verification::test_fastq_gold_repro(workspace, args)
        }
        NativeOpsCommandKey::TestToyRuns => verification::test_toy_runs(workspace, args),
        NativeOpsCommandKey::ToolingCargoTargets => tooling::tooling_cargo_targets(workspace, args),
        NativeOpsCommandKey::ToolingCheckConfigSnapshot => {
            tooling::tooling_check_config_snapshot(workspace, args)
        }
        NativeOpsCommandKey::ToolingCheckConfigPaths => {
            tooling::tooling_check_config_paths(workspace, args)
        }
        NativeOpsCommandKey::ToolingCiAudit => tooling::tooling_ci_audit(workspace, args),
        NativeOpsCommandKey::ToolingCiClippy => tooling::tooling_ci_clippy(workspace, args),
        NativeOpsCommandKey::ToolingCiClippyExecutors => {
            tooling::tooling_ci_clippy_executors(workspace, args)
        }
        NativeOpsCommandKey::ToolingCiCoverage => tooling::tooling_ci_coverage(workspace, args),
        NativeOpsCommandKey::ToolingCiFast => tooling::tooling_ci_fast(workspace, args),
        NativeOpsCommandKey::ToolingCiFmt => tooling::tooling_ci_fmt(workspace, args),
        NativeOpsCommandKey::ToolingCiInstallTools => {
            tooling::tooling_ci_install_tools(workspace, args)
        }
        NativeOpsCommandKey::ToolingCiSlow => tooling::tooling_ci_slow(workspace, args),
        NativeOpsCommandKey::ToolingCiTest => tooling::tooling_ci_test(workspace, args),
        NativeOpsCommandKey::ToolingCiTestSlow => tooling::tooling_ci_test_slow(workspace, args),
        NativeOpsCommandKey::ToolingCleanDocs => tooling::tooling_clean_docs(workspace, args),
        NativeOpsCommandKey::ToolingCertificationGate => {
            tooling::tooling_certification_gate(workspace, args)
        }
        NativeOpsCommandKey::ToolingCertifyLevel1 => {
            tooling::tooling_certify_level1(workspace, args)
        }
        NativeOpsCommandKey::ToolingCertifyAll => tooling::tooling_certify_all(workspace, args),
        NativeOpsCommandKey::ToolingCertifyBam => tooling::tooling_certify_bam(workspace, args),
        NativeOpsCommandKey::ToolingCertifyDomains => {
            tooling::tooling_certify_domains(workspace, args)
        }
        NativeOpsCommandKey::ToolingCertifyFastq => tooling::tooling_certify_fastq(workspace, args),
        NativeOpsCommandKey::ToolingCertifyVcf => tooling::tooling_certify_vcf(workspace, args),
        NativeOpsCommandKey::ToolingAcquireMaps => tooling::tooling_acquire_maps(workspace, args),
        NativeOpsCommandKey::ToolingAcquirePanels => {
            tooling::tooling_acquire_panels(workspace, args)
        }
        NativeOpsCommandKey::ToolingAcquireReference => {
            tooling::tooling_acquire_reference(workspace, args)
        }
        NativeOpsCommandKey::ToolingReferenceExternalData => {
            tooling::tooling_reference_external_data(workspace, args)
        }
        NativeOpsCommandKey::ToolingArchitectureReport => {
            tooling::tooling_architecture_report(workspace, args)
        }
        NativeOpsCommandKey::ToolingBenchmarkSmokeLevel1 => {
            tooling::tooling_benchmark_smoke_level1(workspace, args)
        }
        NativeOpsCommandKey::ToolingBenchmarkIntegrityMini => {
            tooling::tooling_benchmark_integrity_mini(workspace, args)
        }
        NativeOpsCommandKey::ToolingConfigInventory => {
            tooling::tooling_config_inventory(workspace, args)
        }
        NativeOpsCommandKey::ToolingCoverageSummary => {
            tooling::tooling_coverage_summary(workspace, args)
        }
        NativeOpsCommandKey::ToolingCrashTriage => tooling::tooling_crash_triage(workspace, args),
        NativeOpsCommandKey::ToolingDeprecateVcfKnob => {
            tooling::tooling_deprecate_vcf_knob(workspace, args)
        }
        NativeOpsCommandKey::ToolingDeprecateVcfPanel => {
            tooling::tooling_deprecate_vcf_panel(workspace, args)
        }
        NativeOpsCommandKey::ToolingDocsBuild => tooling::tooling_docs_build(workspace, args),
        NativeOpsCommandKey::ToolingFlakeHunt => tooling::tooling_flake_hunt(workspace, args),
        NativeOpsCommandKey::ToolingGenerateConfigs => {
            tooling::tooling_generate_configs(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateCompatibilityMatrix => {
            tooling::tooling_generate_compatibility_matrix(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateConfigTreeSnapshot => {
            tooling::tooling_generate_config_tree_snapshot(workspace, args)
        }
        NativeOpsCommandKey::ToolingGeneratePanelCompatibilityMatrix => {
            tooling::tooling_generate_panel_compatibility_matrix(workspace, args)
        }
        NativeOpsCommandKey::ToolingGeneratePolicyIndex => {
            tooling::tooling_generate_policy_index(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateDocs => tooling::tooling_generate_docs(workspace, args),
        NativeOpsCommandKey::ToolingGenerateDocsGraph => {
            tooling::tooling_generate_docs_graph(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateDomainCoverageDoc => {
            tooling::tooling_generate_domain_coverage_doc(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateRepoRootMap => {
            tooling::tooling_generate_repo_root_map(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateToolIndex => {
            tooling::tooling_generate_tool_index(workspace, args)
        }
        NativeOpsCommandKey::ToolingImageQa => tooling::tooling_image_qa(workspace, args),
        NativeOpsCommandKey::ToolingInventory => tooling::tooling_inventory(workspace, args),
        NativeOpsCommandKey::ToolingLintFast => tooling::tooling_lint_fast(workspace, args),
        NativeOpsCommandKey::ToolingMakeHelp => tooling::tooling_make_help(workspace, args),
        NativeOpsCommandKey::ToolingRepoDoctor => tooling::tooling_repo_doctor(workspace, args),
        NativeOpsCommandKey::ToolingRunBijux => tooling::tooling_run_bijux(workspace, args),
        NativeOpsCommandKey::ToolingSetupDocsVenv => {
            tooling::tooling_setup_docs_venv(workspace, args)
        }
        NativeOpsCommandKey::ToolingSimulateCoverageRegime => {
            tooling::tooling_simulate_coverage_regime(workspace, args)
        }
        NativeOpsCommandKey::ToolingValidateFrontendMiniDomainStacks => {
            tooling::tooling_validate_frontend_mini_domain_stacks(workspace, args)
        }
    }
}
