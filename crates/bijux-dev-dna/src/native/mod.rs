mod containers;
mod domain;
mod script_surface;
mod support;
mod workspace_checks;

use anyhow::Result;

use crate::infrastructure::workspace::Workspace;
use crate::model::check::{CheckDefinition, CheckOutcome, NativeCheckKey};
use crate::model::container::{ContainerCommandOutcome, NativeContainerCommandKey};
use crate::model::domain::{DomainCommandOutcome, NativeDomainCommandKey};

/// # Errors
/// Returns an error if the native check cannot run.
pub fn run_native_check(
    key: &NativeCheckKey,
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    match key {
        NativeCheckKey::AuditAllowlist => workspace_checks::check_audit_allowlist(workspace, check),
        NativeCheckKey::ArtifactEnvContract => {
            workspace_checks::check_artifact_env_contract(workspace, check)
        }
        NativeCheckKey::ArtifactsLayout => {
            workspace_checks::check_artifacts_layout(workspace, check)
        }
        NativeCheckKey::ArtifactsTracked => {
            workspace_checks::check_artifacts_tracked(workspace, check)
        }
        NativeCheckKey::AssetsReferenceSchema => {
            workspace_checks::check_assets_reference_schema(workspace, check)
        }
        NativeCheckKey::BenchKnobDisciplineDownstream => {
            workspace_checks::check_bench_knob_discipline_downstream(workspace, check)
        }
        NativeCheckKey::BenchKnobs => workspace_checks::check_bench_knobs(workspace, check),
        NativeCheckKey::BenchmarkIntegrityPolicy => {
            workspace_checks::check_benchmark_integrity_policy(workspace, check)
        }
        NativeCheckKey::CargoConfigPolicy => {
            workspace_checks::check_cargo_config_policy(workspace, check)
        }
        NativeCheckKey::CertificationSchemaDocs => {
            workspace_checks::check_certification_schema_docs(workspace, check)
        }
        NativeCheckKey::CiShellScripts => script_surface::check_ci_shell_scripts(workspace, check),
        NativeCheckKey::ClippyAllowlistExpiry => {
            workspace_checks::check_clippy_allowlist_expiry(workspace, check)
        }
        NativeCheckKey::ClippyAllowlistGrowth => {
            workspace_checks::check_clippy_allowlist_growth(workspace, check)
        }
        NativeCheckKey::ConfigSchema => workspace_checks::check_config_schema(workspace, check),
        NativeCheckKey::DocsBuildContract => {
            workspace_checks::check_docs_build_contract(workspace, check)
        }
        NativeCheckKey::DocsRequirementsLock => {
            workspace_checks::check_docs_requirements_lock(workspace, check)
        }
        NativeCheckKey::ExamplesRunnerContract => {
            workspace_checks::check_examples_runner_contract(workspace, check)
        }
        NativeCheckKey::ExitCodes => script_surface::check_exit_codes(workspace, check),
        NativeCheckKey::FrontendMiniDomainValidation => {
            workspace_checks::check_frontend_mini_domain_validation(workspace, check)
        }
        NativeCheckKey::GeneratedConfigs => {
            workspace_checks::check_generated_configs(workspace, check)
        }
        NativeCheckKey::GitignoreContract => {
            workspace_checks::check_gitignore_contract(workspace, check)
        }
        NativeCheckKey::HiddenTmpUsage => {
            workspace_checks::check_hidden_tmp_usage(workspace, check)
        }
        NativeCheckKey::HpcSafety => workspace_checks::check_hpc_safety(workspace, check),
        NativeCheckKey::HpcRsyncDocsParity => {
            workspace_checks::check_hpc_rsync_docs_parity(workspace, check)
        }
        NativeCheckKey::LibApi => script_surface::check_lib_api(workspace, check),
        NativeCheckKey::LoggingContract => {
            workspace_checks::check_logging_contract(workspace, check)
        }
        NativeCheckKey::MakeHelpSync => workspace_checks::check_make_help_sync(workspace, check),
        NativeCheckKey::NetworkUsage => script_surface::check_network_usage(workspace, check),
        NativeCheckKey::NoFakeArtifacts => {
            workspace_checks::check_no_fake_artifacts(workspace, check)
        }
        NativeCheckKey::NoOrphanScripts => {
            script_surface::check_no_orphan_scripts(workspace, check)
        }
        NativeCheckKey::NoParallelAccidental => {
            script_surface::check_no_parallel_accidental(workspace, check)
        }
        NativeCheckKey::NoRawCargoInMakes => {
            script_surface::check_no_raw_cargo_in_makes(workspace, check)
        }
        NativeCheckKey::NoRawCargoInScripts => {
            script_surface::check_no_raw_cargo_in_scripts(workspace, check)
        }
        NativeCheckKey::NoTargetPathsInTests => {
            workspace_checks::check_no_target_paths_in_tests(workspace, check)
        }
        NativeCheckKey::NoTempLeaks => script_surface::check_no_temp_leaks(workspace, check),
        NativeCheckKey::NoUserPathLiterals => {
            workspace_checks::check_no_user_path_literals(workspace, check)
        }
        NativeCheckKey::OutputRoots => workspace_checks::check_output_roots(workspace, check),
        NativeCheckKey::ReadmeLinks => workspace_checks::check_readme_links(workspace, check),
        NativeCheckKey::RootLayout => workspace_checks::check_root_layout(workspace, check),
        NativeCheckKey::RuntimeExecutionKernelConfig => {
            workspace_checks::check_runtime_execution_kernel_config(workspace, check)
        }
        NativeCheckKey::RustflagsConsistency => {
            workspace_checks::check_rustflags_consistency(workspace, check)
        }
        NativeCheckKey::ScriptArgStyle => script_surface::check_script_arg_style(workspace, check),
        NativeCheckKey::ScriptDeps => script_surface::check_script_deps(workspace, check),
        NativeCheckKey::ScriptEntrypoint => {
            script_surface::check_script_entrypoint(workspace, check)
        }
        NativeCheckKey::ScriptHelp => script_surface::check_script_help(workspace, check),
        NativeCheckKey::ScriptInterface => script_surface::check_script_interface(workspace, check),
        NativeCheckKey::ScriptWrites => script_surface::check_script_writes(workspace, check),
        NativeCheckKey::ShellPortability => {
            script_surface::check_shell_portability(workspace, check)
        }
        NativeCheckKey::SsotGuardrails => workspace_checks::check_ssot_guardrails(workspace, check),
        NativeCheckKey::SpeciesAliases => workspace_checks::check_species_aliases(workspace, check),
        NativeCheckKey::SupportedScripts => {
            script_surface::check_supported_scripts(workspace, check)
        }
        NativeCheckKey::ToolRegistryLock => {
            workspace_checks::check_tool_registry_lock(workspace, check)
        }
        NativeCheckKey::TreeIntent => script_surface::check_tree_intent(workspace, check),
        NativeCheckKey::VcfCompatibilityMatrix => {
            workspace_checks::check_vcf_compatibility_matrix(workspace, check)
        }
    }
}

/// # Errors
/// Returns an error if the native container command cannot run.
pub fn run_native_container_command(
    key: &NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    containers::run_native_container_command(key, workspace, args)
}

/// # Errors
/// Returns an error if the native domain command cannot run.
pub fn run_native_domain_command(
    key: &NativeDomainCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<DomainCommandOutcome> {
    domain::run_native_domain_command(key, workspace, args)
}
