use anyhow::Result;

use crate::model::check::{CheckDefinition, CheckOutcome, NativeCheckKey};
use crate::runtime::workspace::Workspace;

use super::{automation_boundary, repo_checks};

/// # Errors
/// Returns an error if the native check cannot run.
pub fn run_native_check(
    key: &NativeCheckKey,
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    match key {
        NativeCheckKey::AuditAllowlist => repo_checks::check_audit_allowlist(workspace, check),
        NativeCheckKey::ArtifactEnvContract => {
            repo_checks::check_artifact_env_contract(workspace, check)
        }
        NativeCheckKey::ArtifactsLayout => repo_checks::check_artifacts_layout(workspace, check),
        NativeCheckKey::ArtifactsTracked => repo_checks::check_artifacts_tracked(workspace, check),
        NativeCheckKey::AssetsReferenceSchema => {
            repo_checks::check_assets_reference_schema(workspace, check)
        }
        NativeCheckKey::BenchKnobDisciplineDownstream => {
            repo_checks::check_bench_knob_discipline_downstream(workspace, check)
        }
        NativeCheckKey::BenchKnobs => repo_checks::check_bench_knobs(workspace, check),
        NativeCheckKey::BenchmarkIntegrityPolicy => {
            repo_checks::check_benchmark_integrity_policy(workspace, check)
        }
        NativeCheckKey::CargoConfigPolicy => {
            repo_checks::check_cargo_config_policy(workspace, check)
        }
        NativeCheckKey::CertificationSchemaDocs => {
            repo_checks::check_certification_schema_docs(workspace, check)
        }
        NativeCheckKey::CiAutomationSurface => {
            automation_boundary::check_ci_automation_surface(workspace, check)
        }
        NativeCheckKey::ClippyAllowlistExpiry => {
            repo_checks::check_clippy_allowlist_expiry(workspace, check)
        }
        NativeCheckKey::ClippyAllowlistGrowth => {
            repo_checks::check_clippy_allowlist_growth(workspace, check)
        }
        NativeCheckKey::ConfigSchema => repo_checks::check_config_schema(workspace, check),
        NativeCheckKey::DocsBuildContract => {
            repo_checks::check_docs_build_contract(workspace, check)
        }
        NativeCheckKey::DocsRequirementsLock => {
            repo_checks::check_docs_requirements_lock(workspace, check)
        }
        NativeCheckKey::ExamplesRunnerContract => {
            repo_checks::check_examples_runner_contract(workspace, check)
        }
        NativeCheckKey::AutomationExitCodes => {
            automation_boundary::check_automation_exit_codes(workspace, check)
        }
        NativeCheckKey::FrontendMiniDomainValidation => {
            repo_checks::check_frontend_mini_domain_validation(workspace, check)
        }
        NativeCheckKey::GeneratedConfigs => repo_checks::check_generated_configs(workspace, check),
        NativeCheckKey::GitignoreContract => {
            repo_checks::check_gitignore_contract(workspace, check)
        }
        NativeCheckKey::HiddenTmpUsage => repo_checks::check_hidden_tmp_usage(workspace, check),
        NativeCheckKey::HpcSafety => repo_checks::check_hpc_safety(workspace, check),
        NativeCheckKey::HpcRsyncDocsParity => {
            repo_checks::check_hpc_rsync_docs_parity(workspace, check)
        }
        NativeCheckKey::AutomationBoundary => {
            automation_boundary::check_automation_boundary(workspace, check)
        }
        NativeCheckKey::LoggingContract => repo_checks::check_logging_contract(workspace, check),
        NativeCheckKey::MakeHelpSync => repo_checks::check_make_help_sync(workspace, check),
        NativeCheckKey::AutomationNetworkUsage => {
            automation_boundary::check_automation_network_usage(workspace, check)
        }
        NativeCheckKey::NoFakeArtifacts => repo_checks::check_no_fake_artifacts(workspace, check),
        NativeCheckKey::LegacyAutomationReferences => {
            automation_boundary::check_legacy_automation_references(workspace, check)
        }
        NativeCheckKey::AutomationParallelism => {
            automation_boundary::check_automation_parallelism(workspace, check)
        }
        NativeCheckKey::NoRawCargoInMakes => {
            automation_boundary::check_no_raw_cargo_in_makes(workspace, check)
        }
        NativeCheckKey::NoRawCargoInAutomation => {
            automation_boundary::check_no_raw_cargo_in_automation(workspace, check)
        }
        NativeCheckKey::NoTargetPathsInTests => {
            repo_checks::check_no_target_paths_in_tests(workspace, check)
        }
        NativeCheckKey::AutomationTempDiscipline => {
            automation_boundary::check_automation_temp_discipline(workspace, check)
        }
        NativeCheckKey::NoUserPathLiterals => {
            repo_checks::check_no_user_path_literals(workspace, check)
        }
        NativeCheckKey::OutputRoots => repo_checks::check_output_roots(workspace, check),
        NativeCheckKey::ReadmeLinks => repo_checks::check_readme_links(workspace, check),
        NativeCheckKey::RootLayout => repo_checks::check_root_layout(workspace, check),
        NativeCheckKey::RuntimeExecutionKernelConfig => {
            repo_checks::check_runtime_execution_kernel_config(workspace, check)
        }
        NativeCheckKey::RustflagsConsistency => {
            repo_checks::check_rustflags_consistency(workspace, check)
        }
        NativeCheckKey::AutomationArgStyle => {
            automation_boundary::check_automation_arg_style(workspace, check)
        }
        NativeCheckKey::AutomationDependencies => {
            automation_boundary::check_automation_dependencies(workspace, check)
        }
        NativeCheckKey::AutomationEntrypoints => {
            automation_boundary::check_automation_entrypoints(workspace, check)
        }
        NativeCheckKey::AutomationHelp => {
            automation_boundary::check_automation_help(workspace, check)
        }
        NativeCheckKey::AutomationInterface => {
            automation_boundary::check_automation_interface(workspace, check)
        }
        NativeCheckKey::AutomationWrites => {
            automation_boundary::check_automation_writes(workspace, check)
        }
        NativeCheckKey::AutomationPortability => {
            automation_boundary::check_automation_portability(workspace, check)
        }
        NativeCheckKey::SsotGuardrails => repo_checks::check_ssot_guardrails(workspace, check),
        NativeCheckKey::SpeciesAliases => repo_checks::check_species_aliases(workspace, check),
        NativeCheckKey::LegacyAutomationRemoved => {
            automation_boundary::check_legacy_automation_removed(workspace, check)
        }
        NativeCheckKey::ToolRegistryLock => repo_checks::check_tool_registry_lock(workspace, check),
        NativeCheckKey::AutomationIntent => {
            automation_boundary::check_automation_intent(workspace, check)
        }
        NativeCheckKey::VcfCompatibilityMatrix => {
            repo_checks::check_vcf_compatibility_matrix(workspace, check)
        }
    }
}
