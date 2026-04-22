use crate::model::check::{CheckDefinition, CommandSpec, ExecutionMode, NativeCheckKey};

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn check_registry() -> Vec<CheckDefinition> {
    vec![
        native(
            "check-artifact-env-contract",
            NativeCheckKey::ArtifactEnvContract,
        ),
        native("check-artifacts-layout", NativeCheckKey::ArtifactsLayout),
        native("check-artifacts-tracked", NativeCheckKey::ArtifactsTracked),
        policy("check-asset-checksums", "assets_governance_policy"),
        policy("check-asset-manifests", "assets_governance_policy"),
        policy("check-assets-contracts", "assets_governance_policy"),
        policy("check-assets-drift", "assets_governance_policy"),
        policy("check-assets-large-file-allowlist", "assets_scope_policy"),
        native(
            "check-assets-reference-schema",
            NativeCheckKey::AssetsReferenceSchema,
        ),
        native("check-audit-allowlist", NativeCheckKey::AuditAllowlist),
        native(
            "check-deny-policy-deviations",
            NativeCheckKey::DenyPolicyDeviations,
        ),
        native(
            "check-bench-knob-discipline-downstream",
            NativeCheckKey::BenchKnobDisciplineDownstream,
        ),
        native("check-bench-knobs", NativeCheckKey::BenchKnobs),
        native(
            "check-benchmark-integrity-policy",
            NativeCheckKey::BenchmarkIntegrityPolicy,
        ),
        native(
            "check-cargo-config-policy",
            NativeCheckKey::CargoConfigPolicy,
        ),
        native(
            "check-certification-schema-docs",
            NativeCheckKey::CertificationSchemaDocs,
        ),
        native(
            "check-ci-automation-surface",
            NativeCheckKey::CiAutomationSurface,
        ),
        policy("check-cli-command-snapshot", "cli_command_snapshot_policy"),
        native(
            "check-clippy-allowlist-expiry",
            NativeCheckKey::ClippyAllowlistExpiry,
        ),
        native(
            "check-clippy-allowlist-growth",
            NativeCheckKey::ClippyAllowlistGrowth,
        ),
        policy("check-config-contract-headers", "generated_configs_policy"),
        policy("check-config-filenames", "generated_configs_policy"),
        policy("check-config-headers", "generated_configs_policy"),
        policy("check-config-index-discipline", "generated_configs_policy"),
        policy("check-config-layout", "configs_layout_policy"),
        policy("check-config-owners", "contract_authority_policy"),
        native("check-config-schema", NativeCheckKey::ConfigSchema),
        policy(
            "check-container-ssot-parity",
            "container_registry_parity_policy",
        ),
        policy(
            "check-coverage-regimes-schema",
            "vcf_coverage_regime_policy",
        ),
        policy(
            "check-deprecations-enforcement",
            "tool_registry_reproducibility_policy",
        ),
        native(
            "check-docs-build-contract",
            NativeCheckKey::DocsBuildContract,
        ),
        native(
            "check-docs-requirements-lock",
            NativeCheckKey::DocsRequirementsLock,
        ),
        policy("check-domain-realization", "domain_catalog_symmetry_policy"),
        policy(
            "check-domain-tool-parity",
            "tool_registry_stage_domain_policy",
        ),
        policy(
            "check-enabled-vcf-panel-metadata",
            "vcf_support_gate_policy",
        ),
        composite(
            "check-examples-cli-snapshot",
            &["check-cli-command-snapshot", "check-examples-policy"],
        ),
        policy(
            "check-examples-corpus-checksums",
            "examples_golden_hygiene_policy",
        ),
        policy(
            "check-examples-corpus-layout",
            "examples_cli_command_policy",
        ),
        policy(
            "check-examples-corpus-manifests",
            "examples_golden_hygiene_policy",
        ),
        policy("check-examples-golden", "examples_golden_hygiene_policy"),
        policy("check-examples-index-ssot", "examples_cli_command_policy"),
        policy(
            "check-examples-notebook-policy",
            "examples_cli_command_policy",
        ),
        policy("check-examples-policy", "examples_cli_command_policy"),
        native(
            "check-examples-runner-contract",
            NativeCheckKey::ExamplesRunnerContract,
        ),
        policy("check-examples-structure", "examples_cli_command_policy"),
        policy(
            "check-executor-features-docs",
            "stage_executor_parity_policy",
        ),
        policy("check-executor-no-unwrap", "stage_executor_parity_policy"),
        native(
            "check-automation-exit-codes",
            NativeCheckKey::AutomationExitCodes,
        ),
        policy("check-frontend-mini-artifacts", "artifacts_policy"),
        native(
            "check-frontend-mini-domain-validation",
            NativeCheckKey::FrontendMiniDomainValidation,
        ),
        policy(
            "check-frontend-observability-proof",
            "opentelemetry_version_policy",
        ),
        policy(
            "check-frontend-telemetry-sanity",
            "opentelemetry_version_policy",
        ),
        policy("check-generated-config-headers", "generated_configs_policy"),
        native("check-generated-configs", NativeCheckKey::GeneratedConfigs),
        native(
            "check-gitignore-contract",
            NativeCheckKey::GitignoreContract,
        ),
        policy(
            "check-golden-artifact-schema",
            "examples_golden_hygiene_policy",
        ),
        native("check-hidden-tmp-usage", NativeCheckKey::HiddenTmpUsage),
        process(
            "check-hpc-frontend-constraints",
            "cargo",
            &[
                "run",
                "-q",
                "-p",
                "bijux-dna-dev",
                "--",
                "hpc",
                "run",
                "validate-frontend-constraints",
                "--",
                "--confirm",
            ],
        ),
        native(
            "check-hpc-rsync-docs-parity",
            NativeCheckKey::HpcRsyncDocsParity,
        ),
        native("check-hpc-safety", NativeCheckKey::HpcSafety),
        native(
            "check-automation-boundary",
            NativeCheckKey::AutomationBoundary,
        ),
        native("check-logging-contract", NativeCheckKey::LoggingContract),
        native("check-make-help-sync", NativeCheckKey::MakeHelpSync),
        policy("check-map-locks", "vcf_support_gate_policy"),
        native(
            "check-automation-network-usage",
            NativeCheckKey::AutomationNetworkUsage,
        ),
        policy(
            "check-nextest-profile-contract",
            "nextest_determinism_policy",
        ),
        native("check-no-fake-artifacts", NativeCheckKey::NoFakeArtifacts),
        native(
            "check-legacy-automation-references",
            NativeCheckKey::LegacyAutomationReferences,
        ),
        native(
            "check-no-parallel-accidental",
            NativeCheckKey::AutomationParallelism,
        ),
        native(
            "check-no-raw-cargo-in-makes",
            NativeCheckKey::NoRawCargoInMakes,
        ),
        native(
            "check-no-raw-cargo-in-automation",
            NativeCheckKey::NoRawCargoInAutomation,
        ),
        native(
            "check-no-target-paths-in-tests",
            NativeCheckKey::NoTargetPathsInTests,
        ),
        native(
            "check-automation-temp-discipline",
            NativeCheckKey::AutomationTempDiscipline,
        ),
        native(
            "check-no-user-path-literals",
            NativeCheckKey::NoUserPathLiterals,
        ),
        native("check-output-roots", NativeCheckKey::OutputRoots),
        policy("check-panel-license-policy", "vcf_support_gate_policy"),
        policy("check-panel-locks", "vcf_support_gate_policy"),
        policy(
            "check-param-registry-completeness",
            "contract_authority_policy",
        ),
        native("check-readme-links", NativeCheckKey::ReadmeLinks),
        policy("check-reference-fetch-paths", "error_boundary_policy"),
        policy(
            "check-reference-path-governance",
            "assets_governance_policy",
        ),
        policy("check-reference-service-boundary", "error_boundary_policy"),
        policy(
            "check-registry-required-tools-parity",
            "tool_registry_reproducibility_policy",
        ),
        policy("check-registry-split", "tool_registry_completeness"),
        native("check-root-layout", NativeCheckKey::RootLayout),
        policy("check-run-directory-layout", "root_layout_policy"),
        native(
            "check-runtime-execution-kernel-config",
            NativeCheckKey::RuntimeExecutionKernelConfig,
        ),
        policy("check-runtime-profiles-contract", "profiles_runtime_policy"),
        native(
            "check-rustflags-consistency",
            NativeCheckKey::RustflagsConsistency,
        ),
        native(
            "check-automation-arg-style",
            NativeCheckKey::AutomationArgStyle,
        ),
        native(
            "check-automation-deps",
            NativeCheckKey::AutomationDependencies,
        ),
        native(
            "check-automation-entrypoints",
            NativeCheckKey::AutomationEntrypoints,
        ),
        native("check-automation-help", NativeCheckKey::AutomationHelp),
        native(
            "check-automation-interface",
            NativeCheckKey::AutomationInterface,
        ),
        native("check-automation-writes", NativeCheckKey::AutomationWrites),
        native(
            "check-automation-portability",
            NativeCheckKey::AutomationPortability,
        ),
        native("check-species-aliases", NativeCheckKey::SpeciesAliases),
        native("check-ssot-guardrails", NativeCheckKey::SsotGuardrails),
        composite(
            "check-stage-domain-parity",
            &["check-stage-id-symmetry", "check-stage-executor-parity"],
        ),
        composite(
            "check-stage-registry-governance",
            &["check-registry-ssot", "check-stage-registry-fixtures"],
        ),
        native(
            "check-legacy-automation-removed",
            NativeCheckKey::LegacyAutomationRemoved,
        ),
        native("check-tool-registry-lock", NativeCheckKey::ToolRegistryLock),
        native(
            "check-vcf-compatibility-matrix",
            NativeCheckKey::VcfCompatibilityMatrix,
        ),
        policy("check-vcf-deprecation-lifecycle", "vcf_support_gate_policy"),
        policy("check-vcf-downstream-readiness", "vcf_support_gate_policy"),
        policy("check-vcf-reference-governance", "vcf_support_gate_policy"),
        policy_alias(
            "check-stage-executor-parity",
            "stage_executor_parity_policy",
        ),
        policy_alias("check-stage-id-symmetry", "stage_id_symmetry_policy"),
        policy_alias("check-registry-ssot", "registry_ssot_completeness_policy"),
        policy_alias(
            "check-stage-registry-fixtures",
            "stage_registry_fixture_completeness",
        ),
        CheckDefinition {
            id: "check-automation-intent",
            version: 1,
            summary: "Validate that control-plane surfaces keep a durable purpose.".to_string(),
            aliases: &[],
            execution_mode: ExecutionMode::Alias,
            command: CommandSpec::Native {
                key: NativeCheckKey::AutomationIntent,
            },
        },
    ]
}

fn policy(id: &'static str, filter: &'static str) -> CheckDefinition {
    CheckDefinition {
        id,
        version: 1,
        summary: summary_from_id(id),
        aliases: &[],
        execution_mode: ExecutionMode::Primary,
        command: CommandSpec::CargoTest {
            package: "bijux-dna-policies",
            test_bin: "contracts",
            filter,
        },
    }
}

fn policy_alias(id: &'static str, filter: &'static str) -> CheckDefinition {
    CheckDefinition {
        execution_mode: ExecutionMode::Alias,
        ..policy(id, filter)
    }
}

fn native(id: &'static str, key: NativeCheckKey) -> CheckDefinition {
    CheckDefinition {
        id,
        version: 1,
        summary: summary_from_id(id),
        aliases: &[],
        execution_mode: ExecutionMode::Primary,
        command: CommandSpec::Native { key },
    }
}

fn process(
    id: &'static str,
    program: &'static str,
    args: &'static [&'static str],
) -> CheckDefinition {
    CheckDefinition {
        id,
        version: 1,
        summary: summary_from_id(id),
        aliases: &[],
        execution_mode: ExecutionMode::Primary,
        command: CommandSpec::Process { program, args },
    }
}

fn composite(id: &'static str, members: &'static [&'static str]) -> CheckDefinition {
    CheckDefinition {
        id,
        version: 1,
        summary: summary_from_id(id),
        aliases: &[],
        execution_mode: ExecutionMode::Primary,
        command: CommandSpec::Composite { members },
    }
}

fn summary_from_id(id: &str) -> String {
    let label = id.trim_start_matches("check-").replace('-', " ");
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => format!("{}{}.", first.to_ascii_uppercase(), chars.as_str()),
        None => "Workspace check.".to_string(),
    }
}
