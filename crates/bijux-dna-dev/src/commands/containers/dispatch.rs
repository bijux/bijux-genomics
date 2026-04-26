#![allow(clippy::too_many_lines)]

#[allow(clippy::wildcard_imports)]
use super::*;

pub(super) fn run_native_container_command(
    key: NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    match &key {
        NativeContainerCommandKey::Lint => validation::run_container_lint(workspace, args),
        NativeContainerCommandKey::RegistryTools => validation::run_registry_tools(workspace, args),
        NativeContainerCommandKey::EnsureImages => validation::run_ensure_images(workspace, args),
        NativeContainerCommandKey::ContainerDoctor => {
            validation::run_container_doctor(workspace, args)
        }
        NativeContainerCommandKey::ReleaseGate => validation::run_release_gate(workspace, args),
        NativeContainerCommandKey::VulnScanHook => validation::run_vuln_scan_hook(workspace, args),
        NativeContainerCommandKey::ApptainerBuildAll => {
            validation::run_apptainer_build_all(workspace, args)
        }
        NativeContainerCommandKey::BuildApptainerAll => {
            validation::run_build_apptainer_all(workspace, args)
        }
        NativeContainerCommandKey::BuildApptainerHpcFrontend => {
            validation::run_build_apptainer_hpc_frontend(workspace, args)
        }
        NativeContainerCommandKey::DockerBuildAll => {
            validation::run_docker_build_all(workspace, args)
        }
        NativeContainerCommandKey::SmokeApptainer => {
            ensure_no_args("smoke-apptainer", args)?;
            run_runtime_smoke_contract(workspace, "apptainer", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeDockerAmd64 => {
            ensure_no_args("smoke-docker-amd64", args)?;
            run_runtime_smoke_contract(workspace, "docker-amd64", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeDockerArm64 => {
            ensure_no_args("smoke-docker-arm64", args)?;
            run_runtime_smoke_contract(workspace, "docker-arm64", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::RunApptainerFrontendSmoke => {
            validation::run_apptainer_frontend_smoke(workspace, args)
        }
        NativeContainerCommandKey::RunApptainerFrontendSecurity => {
            validation::run_apptainer_frontend_security(workspace, args)
        }
        NativeContainerCommandKey::RunApptainerFrontendReproducibility => {
            validation::run_apptainer_frontend_reproducibility(workspace, args)
        }
        NativeContainerCommandKey::ContainerRuntimeCheck => {
            ensure_no_args("container-runtime-check", args)?;
            run_container_runtime_check()
        }
        NativeContainerCommandKey::GenerateToolIds => metadata::generate_tool_ids(workspace, args),
        NativeContainerCommandKey::CheckToolIdManifest => {
            ensure_no_args("check-tool-id-manifest", args)?;
            metadata::check_tool_id_manifest(workspace)
        }
        NativeContainerCommandKey::GenerateToolNameMap => {
            metadata::generate_tool_name_map(workspace, args)
        }
        NativeContainerCommandKey::CheckToolNameMapGenerated => {
            ensure_no_args("check-tool-name-map-generated", args)?;
            metadata::check_tool_name_map_generated(workspace)
        }
        NativeContainerCommandKey::GenerateContainerIndex => {
            metadata::generate_container_index(workspace, args)
        }
        NativeContainerCommandKey::CheckContainerIndex => {
            ensure_no_args("check-index", args)?;
            metadata::check_container_index(workspace)
        }
        NativeContainerCommandKey::GenerateGhcrPublishMatrix => {
            metadata::generate_ghcr_publish_matrix(workspace, args)
        }
        NativeContainerCommandKey::GenerateGhcrApptainerPublishMatrix => {
            metadata::generate_ghcr_apptainer_publish_matrix(workspace, args)
        }
        NativeContainerCommandKey::GenerateLicenseMetadata => {
            metadata::generate_license_metadata(workspace, args)
        }
        NativeContainerCommandKey::CheckLicenseMetadata => {
            ensure_no_args("check-license-metadata", args)?;
            metadata::check_license_metadata(workspace)
        }
        NativeContainerCommandKey::CheckLicenseIndexGenerated => {
            ensure_no_args("check-license-index-generated", args)?;
            metadata::check_license_index_generated(workspace)
        }
        NativeContainerCommandKey::GenerateQaMatrix => {
            metadata::generate_qa_matrix(workspace, args)
        }
        NativeContainerCommandKey::CheckQaMatrixGenerated => {
            ensure_no_args("check-qa-matrix-generated", args)?;
            metadata::check_qa_matrix_generated(workspace)
        }
        NativeContainerCommandKey::GenerateToolDocs => {
            metadata::generate_tool_docs(workspace, args)
        }
        NativeContainerCommandKey::CheckToolDocsGenerated => {
            ensure_no_args("check-tool-docs-generated", args)?;
            metadata::check_tool_docs_generated(workspace)
        }
        NativeContainerCommandKey::GenerateNetworkUsage => {
            metadata::generate_network_usage(workspace, args)
        }
        NativeContainerCommandKey::CheckNetworkDisclosure => {
            metadata::check_network_disclosure(workspace, args)
        }
        NativeContainerCommandKey::ExtractVersionMap => {
            versioning::extract_version_map(workspace, args)
        }
        NativeContainerCommandKey::GenerateVersionLock => {
            versioning::generate_version_lock(workspace, args)
        }
        NativeContainerCommandKey::CheckVersionLock => {
            ensure_no_args("check-version-lock", args)?;
            versioning::check_version_lock(workspace)
        }
        NativeContainerCommandKey::CheckVersionAuthority => {
            ensure_no_args("check-version-authority", args)?;
            versioning::check_version_authority(workspace)
        }
        NativeContainerCommandKey::GenerateVersionsIndexSha => {
            versioning::generate_versions_index_sha(workspace, args)
        }
        NativeContainerCommandKey::CheckVersionsIndexSha => {
            ensure_no_args("check-versions-index-sha", args)?;
            versioning::check_versions_index_sha(workspace)
        }
        NativeContainerCommandKey::CheckLockChangeDiscipline => {
            ensure_no_args("check-lock-change-discipline", args)?;
            versioning::check_lock_change_discipline(workspace)
        }
        NativeContainerCommandKey::CheckLockDrift => {
            ensure_no_args("check-lock-drift", args)?;
            versioning::check_version_lock(workspace)
        }
        NativeContainerCommandKey::CheckLockSchema => {
            ensure_no_args("check-lock-schema", args)?;
            versioning::check_lock_schema(workspace)
        }
        NativeContainerCommandKey::CheckVersionCompleteness => {
            ensure_no_args("check-version-completeness", args)?;
            versioning::check_version_completeness(workspace)
        }
        NativeContainerCommandKey::CheckVersionHashPin => {
            ensure_no_args("check-version-hash-pin", args)?;
            versioning::check_version_hash_pin(workspace)
        }
        NativeContainerCommandKey::CheckVersionImmutability => {
            ensure_no_args("check-version-immutability", args)?;
            versioning::check_version_immutability(workspace)
        }
        NativeContainerCommandKey::CheckVersionDeprecations => {
            ensure_no_args("check-version-deprecations", args)?;
            versioning::check_version_deprecations(workspace)
        }
        NativeContainerCommandKey::CheckPromotionPolicy => {
            ensure_no_args("check-promotion-policy", args)?;
            versioning::check_promotion_policy(workspace)
        }
        NativeContainerCommandKey::CheckPromotionLockIntegrity => {
            ensure_no_args("check-promotion-lock-integrity", args)?;
            versioning::check_promotion_lock_integrity(workspace)
        }
        NativeContainerCommandKey::Promote => versioning::promote_tool(workspace, args),
        NativeContainerCommandKey::Demote => versioning::demote_tool(workspace, args),
        NativeContainerCommandKey::DeprecateVersion => {
            versioning::deprecate_version(workspace, args)
        }
        NativeContainerCommandKey::ToolLifecycle => versioning::tool_lifecycle(workspace, args),
        NativeContainerCommandKey::CheckApptainerCachePolicy => {
            ensure_no_args("check-apptainer-cache-policy", args)?;
            check_apptainer_cache_policy(workspace)
        }
        NativeContainerCommandKey::CheckApptainerFrontendReproducibility => {
            check_apptainer_frontend_reproducibility(workspace, args)
        }
        NativeContainerCommandKey::CheckApptainerFrontendSecurity => {
            check_apptainer_frontend_security(workspace, args)
        }
        NativeContainerCommandKey::CheckApptainerFrontendSmokeProof => {
            check_apptainer_frontend_smoke_proof(workspace, args)
        }
        NativeContainerCommandKey::CheckApptainerFrontendVersionOutputLock => {
            ensure_no_args("check-apptainer-frontend-version-output-lock", args)?;
            check_apptainer_frontend_version_output_lock(workspace)
        }
        NativeContainerCommandKey::CheckApptainerHardening => {
            ensure_no_args("check-apptainer-hardening", args)?;
            check_apptainer_hardening(workspace)
        }
        NativeContainerCommandKey::CheckApptainerPostPins => {
            ensure_no_args("check-apptainer-post-pins", args)?;
            check_apptainer_post_pins(workspace)
        }
        NativeContainerCommandKey::CheckApptainerVersionLabelSync => {
            ensure_no_args("check-apptainer-version-label-sync", args)?;
            check_apptainer_version_label_sync(workspace)
        }
        NativeContainerCommandKey::CheckBijuxApptainerBuilt => {
            ensure_no_args("check-bijux-apptainer-built", args)?;
            check_bijux_apptainer_built(workspace)
        }
        NativeContainerCommandKey::GenerateLocalApptainerDigests => {
            generate_local_apptainer_digests(workspace, args)
        }
        NativeContainerCommandKey::CompareFrontendLocalSifHash => {
            compare_frontend_local_sif_hash(workspace, args)
        }
        NativeContainerCommandKey::CheckMissingImages => {
            ensure_no_args("check-missing-images", args)?;
            check_missing_images(workspace)
        }
        NativeContainerCommandKey::CheckNonBijuxSources => {
            ensure_no_args("check-non-bijux-sources", args)?;
            check_non_bijux_sources(workspace)
        }
        NativeContainerCommandKey::CheckOwners => {
            ensure_no_args("check-owners", args)?;
            check_owners(workspace)
        }
        NativeContainerCommandKey::CheckRegistryVsDefs => {
            ensure_no_args("check-registry-vs-defs", args)?;
            check_registry_vs_defs(workspace)
        }
        NativeContainerCommandKey::CheckToolNameCollision => {
            ensure_no_args("check-tool-name-collision", args)?;
            validation::check_tool_name_collision(workspace)
        }
        NativeContainerCommandKey::CheckToolContainerCoverage => {
            ensure_no_args("check-tool-container-coverage", args)?;
            validation::check_tool_container_coverage(workspace)
        }
        NativeContainerCommandKey::CheckToolkitBundles => {
            ensure_no_args("check-toolkit-bundles", args)?;
            validation::check_toolkit_bundles(workspace)
        }
        NativeContainerCommandKey::CheckHpcImageNaming => {
            validation::check_hpc_image_naming(workspace, args)
        }
        NativeContainerCommandKey::CheckPlannedActionability => {
            ensure_no_args("check-planned-actionability", args)?;
            validation::check_planned_actionability(workspace)
        }
        NativeContainerCommandKey::CheckBijuxTemplateMarkers => {
            ensure_no_args("check-bijux-template-markers", args)?;
            validation::check_bijux_template_markers(workspace)
        }
        NativeContainerCommandKey::CheckToolIdContract => {
            ensure_no_args("check-tool-id-contract", args)?;
            validation::check_tool_id_contract(workspace)
        }
        NativeContainerCommandKey::CheckDockerArchPolicy => {
            ensure_no_args("check-docker-arch-policy", args)?;
            validation::check_docker_arch_policy(workspace)
        }
        NativeContainerCommandKey::CheckDockerArm64Completeness => {
            ensure_no_args("check-docker-arm64-completeness", args)?;
            validation::check_docker_arm64_completeness(workspace)
        }
        NativeContainerCommandKey::CheckDockerContext => {
            ensure_no_args("check-docker-context", args)?;
            validation::check_docker_context(workspace)
        }
        NativeContainerCommandKey::CheckDockerHardening => {
            ensure_no_args("check-docker-hardening", args)?;
            validation::check_docker_hardening(workspace)
        }
        NativeContainerCommandKey::CheckDockerLabels => {
            ensure_no_args("check-docker-labels", args)?;
            validation::check_docker_labels(workspace)
        }
        NativeContainerCommandKey::CheckDockerUnpinnedApt => {
            ensure_no_args("check-docker-unpinned-apt", args)?;
            validation::check_docker_unpinned_apt(workspace)
        }
        NativeContainerCommandKey::CheckDockerVersionSync => {
            ensure_no_args("check-docker-version-sync", args)?;
            validation::check_docker_version_sync(workspace)
        }
        NativeContainerCommandKey::CheckDockerfilesBuilt => {
            ensure_no_args("check-dockerfiles-built", args)?;
            validation::check_dockerfiles_built(workspace)
        }
        NativeContainerCommandKey::CheckNoSecrets => {
            ensure_no_args("check-no-secrets", args)?;
            validation::check_no_secrets(workspace)
        }
        NativeContainerCommandKey::CheckRuntimeDownloads => {
            ensure_no_args("check-runtime-downloads", args)?;
            validation::check_runtime_downloads(workspace)
        }
        NativeContainerCommandKey::CheckVulnAllowlist => {
            ensure_no_args("check-vuln-allowlist", args)?;
            validation::check_vuln_allowlist(workspace)
        }
        NativeContainerCommandKey::CheckVulnHook => {
            ensure_no_args("check-vuln-hook", args)?;
            validation::check_vuln_hook(workspace)
        }
        NativeContainerCommandKey::CheckSbomArtifacts => {
            ensure_no_args("check-sbom-artifacts", args)?;
            validation::check_sbom_artifacts(workspace)
        }
        NativeContainerCommandKey::CheckTimeLocaleDeterminism => {
            ensure_no_args("check-time-locale-determinism", args)?;
            validation::check_time_locale_determinism(workspace)
        }
        NativeContainerCommandKey::CheckToolInvocationNormalization => {
            ensure_no_args("check-tool-invocation-normalization", args)?;
            validation::check_tool_invocation_normalization(workspace)
        }
        NativeContainerCommandKey::CheckSmokeInputsPolicy => {
            ensure_no_args("check-smoke-inputs-policy", args)?;
            validation::check_smoke_inputs_policy(workspace)
        }
        NativeContainerCommandKey::CheckCrossRuntimeRepresentative => {
            ensure_no_args("check-cross-runtime-representative", args)?;
            validation::check_cross_runtime_representative(workspace)
        }
        NativeContainerCommandKey::CheckCrossRuntimeSmoke => {
            ensure_no_args("check-cross-runtime-smoke", args)?;
            validation::check_cross_runtime_smoke(workspace)
        }
        NativeContainerCommandKey::CheckSmokeFailureClassification => {
            ensure_no_args("check-smoke-failure-classification", args)?;
            validation::check_smoke_failure_classification(workspace)
        }
        NativeContainerCommandKey::CheckSmokeContract => {
            ensure_no_args("check-smoke-contract", args)?;
            validation::check_smoke_contract(workspace)
        }
        NativeContainerCommandKey::CheckSmokeContractLock => {
            ensure_no_args("check-smoke-contract-lock", args)?;
            validation::check_smoke_contract_lock(workspace)
        }
        NativeContainerCommandKey::CheckVcfImputationToolchain => {
            ensure_no_args("check-vcf-imputation-toolchain", args)?;
            validation::check_vcf_imputation_toolchain(workspace)
        }
        NativeContainerCommandKey::CheckImputationRuntimeConstraints => {
            ensure_no_args("check-imputation-runtime-constraints", args)?;
            validation::check_imputation_runtime_constraints(workspace)
        }
        NativeContainerCommandKey::CheckImputationNetworkPolicy => {
            ensure_no_args("check-imputation-network-policy", args)?;
            validation::check_imputation_network_policy(workspace)
        }
        NativeContainerCommandKey::CheckImputationHardening => {
            ensure_no_args("check-imputation-hardening", args)?;
            validation::check_imputation_hardening(workspace)
        }
        NativeContainerCommandKey::CheckImputationReleaseSmoke => {
            ensure_no_args("check-imputation-release-smoke", args)?;
            validation::check_imputation_release_smoke(workspace)
        }
        NativeContainerCommandKey::CheckImputationCrossRuntimeParity => {
            ensure_no_args("check-imputation-cross-runtime-parity", args)?;
            validation::check_imputation_cross_runtime_parity(workspace)
        }
        NativeContainerCommandKey::CheckBuildProvenance => {
            ensure_no_args("check-build-provenance", args)?;
            validation::check_build_provenance(workspace)
        }
        NativeContainerCommandKey::CheckDigestChangesOnVersionChange => {
            ensure_no_args("check-digest-changes-on-version-change", args)?;
            validation::check_digest_changes_on_version_change(workspace)
        }
        NativeContainerCommandKey::CheckDigestOutputPolicy => {
            ensure_no_args("check-digest-output-policy", args)?;
            validation::check_digest_output_policy(workspace)
        }
        NativeContainerCommandKey::CheckRuntimeToolDigestRecording => {
            ensure_no_args("check-runtime-tool-digest-recording", args)?;
            validation::check_runtime_tool_digest_recording(workspace)
        }
        NativeContainerCommandKey::CheckRebuildRepro => {
            validation::check_rebuild_repro(workspace, args)
        }
        NativeContainerCommandKey::CheckApptainerRebuildRepro => {
            validation::check_apptainer_rebuild_repro(workspace, args)
        }
        NativeContainerCommandKey::CheckApptainerBijuxHeader => {
            ensure_no_args("check-apptainer-bijux-header", args)?;
            validation::check_apptainer_bijux_header(workspace)
        }
        NativeContainerCommandKey::CheckHpcFrontendPolicyEnforcement => {
            ensure_no_args("check-hpc-frontend-policy-enforcement", args)?;
            validation::check_hpc_frontend_policy_enforcement(workspace)
        }
        NativeContainerCommandKey::CheckImageSizeRegression => {
            ensure_no_args("check-image-size-regression", args)?;
            validation::check_image_size_regression(workspace)
        }
        NativeContainerCommandKey::CheckLockMatchesBuiltOutput => {
            ensure_no_args("check-lock-matches-built-output", args)?;
            validation::check_lock_matches_built_output(workspace)
        }
        NativeContainerCommandKey::CheckReleaseChecklist => {
            ensure_no_args("check-release-checklist", args)?;
            validation::check_release_checklist(workspace)
        }
        NativeContainerCommandKey::CheckToolkitBundleBuildable => {
            ensure_no_args("check-toolkit-bundle-buildable", args)?;
            validation::check_toolkit_bundle_buildable(workspace)
        }
        NativeContainerCommandKey::CheckVcfDownstreamBundleCoverage => {
            ensure_no_args("check-vcf-downstream-bundle-coverage", args)?;
            validation::check_vcf_downstream_bundle_coverage(workspace)
        }
        NativeContainerCommandKey::Summary => validation::summary(workspace, args),
        NativeContainerCommandKey::EnvPrep => validation::run_env_prep(workspace, args),
        NativeContainerCommandKey::EnvSmoke => validation::run_env_smoke(workspace, args),
        NativeContainerCommandKey::ContainerSmoke => {
            validation::run_container_smoke(workspace, args)
        }
        NativeContainerCommandKey::ContainersSmoke => {
            validation::run_containers_smoke(workspace, args)
        }
        NativeContainerCommandKey::SmokeContainersDockerArm64 => {
            ensure_no_args("smoke-containers-docker-arm64", args)?;
            run_runtime_smoke_contract(workspace, "docker-arm64", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeContainersDockerAmd64 => {
            ensure_no_args("smoke-containers-docker-amd64", args)?;
            run_runtime_smoke_contract(workspace, "docker-amd64", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeContainersApptainer => {
            ensure_no_args("smoke-containers-apptainer", args)?;
            run_runtime_smoke_contract(workspace, "apptainer", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeContainersApptainerBijuxRun => {
            ensure_no_args("smoke-containers-apptainer-bijux-run", args)?;
            run_runtime_smoke_contract(workspace, "apptainer", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeContainersApptainerApptainerRun => {
            ensure_no_args("smoke-containers-apptainer-apptainer-run", args)?;
            run_runtime_smoke_contract(workspace, "apptainer", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeContainersApptainerVerify => {
            ensure_no_args("smoke-containers-apptainer-verify", args)?;
            compare_apptainer_smoke_modes(Path::new(&container_artifact_dir()))
        }
        NativeContainerCommandKey::SmokeCrossRuntimeVerify => {
            ensure_no_args("smoke-cross-runtime-verify", args)?;
            validation::check_cross_runtime_smoke_at_paths(
                workspace,
                PathBuf::from(format!("{}/docker-arm64", container_artifact_dir())),
                PathBuf::from(format!("{}/apptainer", container_artifact_dir())),
            )
        }
        NativeContainerCommandKey::SmokeToolkitDockerArm64 => {
            ensure_no_args("smoke-toolkit-docker-arm64", args)?;
            let toolkit = required_env("TOOLKIT")?;
            let tools = resolve_toolkit_tools(workspace, &toolkit)?;
            run_runtime_smoke_contract(workspace, "docker-arm64", tools)
        }
        NativeContainerCommandKey::SmokeToolkitApptainer => {
            ensure_no_args("smoke-toolkit-apptainer", args)?;
            let toolkit = required_env("TOOLKIT")?;
            let tools = resolve_toolkit_tools(workspace, &toolkit)?;
            run_runtime_smoke_contract(workspace, "apptainer", tools)
        }
        NativeContainerCommandKey::BuildImages => {
            ensure_no_args("build-images", args)?;
            let tools = if env_or_empty("TOOLS").is_empty() {
                primary_tools_csv(workspace)?
            } else {
                env_or_empty("TOOLS")
            };
            validation::run_build_contract(workspace, &tools)
        }
        NativeContainerCommandKey::BuildTool => {
            ensure_no_args("build-tool", args)?;
            validation::run_build_contract(workspace, &required_env("TOOLS")?)
        }
        NativeContainerCommandKey::BuildAll => {
            ensure_no_args("build-all", args)?;
            validation::run_build_contract(workspace, &primary_tools_csv(workspace)?)
        }
        NativeContainerCommandKey::BuildBundle => {
            ensure_no_args("build-bundle", args)?;
            let toolkit = required_env("TOOLKIT")?;
            validation::run_build_contract(workspace, &resolve_toolkit_tools(workspace, &toolkit)?)
        }
        NativeContainerCommandKey::TestImages => validation::run_test_images(workspace, args),
        NativeContainerCommandKey::TestImagesStage => {
            validation::run_test_images_stage(workspace, args)
        }
        NativeContainerCommandKey::TestImagesTool => {
            validation::run_test_images_tool(workspace, args)
        }
        NativeContainerCommandKey::ImageSmokeVcf => {
            validation::run_image_smoke_vcf(workspace, args)
        }
        NativeContainerCommandKey::ImageQa => validation::run_image_qa(workspace, args),
        NativeContainerCommandKey::ApptainerEnsure => {
            validation::run_apptainer_ensure(workspace, args)
        }
        NativeContainerCommandKey::ApptainerEnsureStage => {
            validation::run_apptainer_ensure_stage(workspace, args)
        }
    }
}
