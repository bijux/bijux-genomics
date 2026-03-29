use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate, Utc};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::model::container::{ContainerCommandOutcome, NativeContainerCommandKey};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

mod metadata;
mod runtime;
mod validation;
mod versioning;

use self::runtime::*;

pub fn run_native_container_command(
    key: &NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    match key {
        NativeContainerCommandKey::Lint => validation::run_container_lint(workspace, args),
        NativeContainerCommandKey::RegistryTools => validation::run_registry_tools(workspace, args),
        NativeContainerCommandKey::EnsureImages => validation::run_ensure_images(workspace, args),
        NativeContainerCommandKey::ContainerDoctor => validation::run_container_doctor(workspace, args),
        NativeContainerCommandKey::ReleaseGate => validation::run_release_gate(workspace, args),
        NativeContainerCommandKey::VulnScanHook => validation::run_vuln_scan_hook(workspace, args),
        NativeContainerCommandKey::ApptainerBuildAll => validation::run_apptainer_build_all(workspace, args),
        NativeContainerCommandKey::BuildApptainerAll => validation::run_build_apptainer_all(workspace, args),
        NativeContainerCommandKey::BuildApptainerHpcFrontend => {
            validation::run_build_apptainer_hpc_frontend(workspace, args)
        }
        NativeContainerCommandKey::DockerBuildAll => validation::run_docker_build_all(workspace, args),
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
        NativeContainerCommandKey::GenerateQaMatrix => metadata::generate_qa_matrix(workspace, args),
        NativeContainerCommandKey::CheckQaMatrixGenerated => {
            ensure_no_args("check-qa-matrix-generated", args)?;
            metadata::check_qa_matrix_generated(workspace)
        }
        NativeContainerCommandKey::GenerateToolDocs => metadata::generate_tool_docs(workspace, args),
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
        NativeContainerCommandKey::ExtractVersionMap => versioning::extract_version_map(workspace, args),
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
        NativeContainerCommandKey::DeprecateVersion => versioning::deprecate_version(workspace, args),
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
        NativeContainerCommandKey::CheckHpcImageNaming => validation::check_hpc_image_naming(workspace, args),
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
        NativeContainerCommandKey::CheckRebuildRepro => validation::check_rebuild_repro(workspace, args),
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
        NativeContainerCommandKey::ContainerSmoke => validation::run_container_smoke(workspace, args),
        NativeContainerCommandKey::ContainersSmoke => validation::run_containers_smoke(workspace, args),
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
        NativeContainerCommandKey::SmokeCntainersApptainerBijuxRun => {
            ensure_no_args("smoke-cntainers-apptainer-bijux-run", args)?;
            run_runtime_smoke_contract(workspace, "apptainer", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeCntainersApptainerApptainerRun => {
            ensure_no_args("smoke-cntainers-apptainer-apptainer-run", args)?;
            run_runtime_smoke_contract(workspace, "apptainer", resolved_smoke_tools(workspace)?)
        }
        NativeContainerCommandKey::SmokeCntainersApptainerVerify => {
            ensure_no_args("smoke-cntainers-apptainer-verify", args)?;
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
        NativeContainerCommandKey::TestImagesStage => validation::run_test_images_stage(workspace, args),
        NativeContainerCommandKey::TestImagesTool => validation::run_test_images_tool(workspace, args),
        NativeContainerCommandKey::ImageSmokeVcf => validation::run_image_smoke_vcf(workspace, args),
        NativeContainerCommandKey::ImageQa => validation::run_image_qa(workspace, args),
        NativeContainerCommandKey::ApptainerEnsure => validation::run_apptainer_ensure(workspace, args),
        NativeContainerCommandKey::ApptainerEnsureStage => {
            validation::run_apptainer_ensure_stage(workspace, args)
        }
    }
}

fn run_container_runtime_check() -> Result<ContainerCommandOutcome> {
    let system_type = std::env::var("SYSTEM_TYPE").unwrap_or_else(|_| "local".to_string());
    let container_type = checked_container_type()?;
    Ok(ContainerCommandOutcome::success(format!(
        "SYSTEM_TYPE={system_type} CONTAINER_TYPE={container_type}\n"
    )))
}

fn success_line(line: impl Into<String>) -> Result<ContainerCommandOutcome> {
    Ok(ContainerCommandOutcome::success(format!(
        "{}\n",
        line.into()
    )))
}

fn failure_lines(title: &str, errors: &[String]) -> Result<ContainerCommandOutcome> {
    let mut stderr = String::new();
    stderr.push_str(title);
    stderr.push('\n');
    for error in errors {
        stderr.push_str(error);
        if !error.ends_with('\n') {
            stderr.push('\n');
        }
    }
    Ok(ContainerCommandOutcome::failure(stderr))
}

fn read_utf8(path: &std::path::Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn write_utf8(path: &std::path::Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(path, content).with_context(|| format!("write {}", path.display()))
}

fn append_named_outcome(
    aggregate: &mut ContainerCommandOutcome,
    name: &str,
    outcome: ContainerCommandOutcome,
) {
    aggregate.stdout.push_str(&format!("== {name}\n"));
    *aggregate = merge_outcomes(aggregate.clone(), outcome);
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_container_label_keys() -> [&'static str; 7] {
    [
        "org.opencontainers.image.source",
        "org.opencontainers.image.revision",
        "org.opencontainers.image.created",
        "org.opencontainers.image.licenses",
        "org.opencontainers.image.version",
        "org.opencontainers.image.tool",
        "org.opencontainers.image.title",
    ]
}

fn missing_container_label_markers(text: &str) -> Vec<&'static str> {
    canonical_container_label_keys()
        .into_iter()
        .filter(|label| !text.contains(label))
        .collect()
}

fn docker_image_labels(workspace: &Workspace, image: &str) -> Result<BTreeMap<String, String>> {
    let inspect = run_program_with_env(
        workspace,
        "docker",
        &[
            "image".to_string(),
            "inspect".to_string(),
            image.to_string(),
            "--format".to_string(),
            "{{json .Config.Labels}}".to_string(),
        ],
        &[],
    )?;
    if !inspect.is_success() {
        return Err(anyhow!(
            "docker image inspect failed for {image}: {}",
            inspect.stderr.trim()
        ));
    }
    let stdout = inspect.stdout.trim();
    if stdout.is_empty() || stdout == "null" {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str(stdout).with_context(|| format!("parse docker labels for {image}"))
}

fn canonical_metadata_labels(labels: &BTreeMap<String, String>) -> BTreeMap<&'static str, String> {
    canonical_container_label_keys()
        .into_iter()
        .filter_map(|key| labels.get(key).cloned().map(|value| (key, value)))
        .collect()
}

fn load_toml(path: &std::path::Path) -> Result<toml::Value> {
    toml::from_str(&read_utf8(path)?).with_context(|| format!("parse TOML {}", path.display()))
}

fn registry_tool_rows(workspace: &Workspace) -> Result<Vec<toml::map::Map<String, toml::Value>>> {
    let mut rows = Vec::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let value = load_toml(&workspace.path(rel))?;
        if let Some(entries) = value.get("tools").and_then(toml::Value::as_array) {
            for entry in entries {
                if let Some(table) = entry.as_table() {
                    rows.push(table.clone());
                }
            }
        }
    }
    Ok(rows)
}

fn registry_tool_map(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let mut rows = BTreeMap::new();
    for row in registry_tool_rows(workspace)? {
        let tool_id = row
            .get("id")
            .or_else(|| row.get("tool_id"))
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !tool_id.is_empty() {
            rows.insert(tool_id, row);
        }
    }
    Ok(rows)
}

fn governed_container_file_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/docker/arm64").display()
            )
        })?
        .filter_map(std::result::Result::ok)
    {
        if let Some(tool_id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix("Dockerfile."))
        {
            ids.insert(tool_id.to_string());
        }
    }
    for entry in fs::read_dir(workspace.path("containers/apptainer/shared"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/apptainer/shared").display()
            )
        })?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("def") {
            if let Some(tool_id) = path.file_stem().and_then(|name| name.to_str()) {
                ids.insert(tool_id.to_string());
            }
        }
    }
    Ok(ids)
}

fn governed_container_statuses(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let registry = registry_tool_map(workspace)?;
    let versions = tool_versions(workspace)?;
    let mut statuses = BTreeMap::new();
    for tool_id in governed_container_file_ids(workspace)? {
        let status = registry
            .get(&tool_id)
            .map(|row| table_string(row, "status"))
            .filter(|value| !value.is_empty())
            .or_else(|| {
                versions
                    .get(&tool_id)
                    .map(|row| table_string(row, "status"))
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| "experimental".to_string());
        statuses.insert(tool_id, status);
    }
    for (tool_id, row) in registry {
        let status = table_string(&row, "status");
        if !status.is_empty() {
            statuses.entry(tool_id).or_insert(status);
        }
    }
    Ok(statuses)
}

fn is_non_bijux_apptainer_source(workspace: &Workspace, tool_id: &str) -> bool {
    let apptainer = workspace.path(&format!("containers/apptainer/shared/{tool_id}.def"));
    apptainer.exists()
        && (read_utf8(&apptainer)
            .unwrap_or_default()
            .contains("NON_BIJUX_SOURCES.md")
            || matches!(
                tool_id,
                "bcftools"
                    | "beagle"
                    | "eagle"
                    | "eigensoft"
                    | "germline"
                    | "glimpse"
                    | "ibdhap"
                    | "ibdne"
                    | "impute5"
                    | "minimac4"
                    | "shapeit5"
            ))
}

fn tool_versions(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let value = load_toml(&workspace.path("containers/versions/versions.toml"))?;
    let Some(table) = value.as_table() else {
        return Ok(BTreeMap::new());
    };
    let mut rows = BTreeMap::new();
    for (tool, entry) in table {
        if let Some(entry_table) = entry.as_table() {
            rows.insert(tool.clone(), entry_table.clone());
        }
    }
    Ok(rows)
}

fn versions_toml_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/versions.toml")
}

fn container_version_deprecations_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/deprecations.toml")
}

fn registry_deprecations_path(workspace: &Workspace) -> PathBuf {
    workspace.path("configs/ci/registry/deprecations.toml")
}

fn lock_json_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/lock.json")
}

fn production_registry_paths(workspace: &Workspace) -> Vec<PathBuf> {
    vec![
        workspace.path("configs/ci/registry/tool_registry.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    ]
}

fn all_registry_paths(workspace: &Workspace) -> Vec<PathBuf> {
    vec![
        workspace.path("configs/ci/registry/tool_registry.toml"),
        workspace.path("configs/ci/registry/tool_registry_experimental.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    ]
}

fn read_lock_json(workspace: &Workspace) -> Result<serde_json::Value> {
    serde_json::from_str(&read_utf8(&lock_json_path(workspace))?)
        .with_context(|| "parse lock.json".to_string())
}

fn lock_items_by_tool(workspace: &Workspace) -> Result<BTreeMap<String, serde_json::Value>> {
    let mut rows = BTreeMap::new();
    if let Some(items) = read_lock_json(workspace)?
        .get("items")
        .and_then(serde_json::Value::as_array)
    {
        for row in items {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !tool.is_empty() {
                rows.insert(tool, row.clone());
            }
        }
    }
    Ok(rows)
}

fn parse_date(value: &str, field_name: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("invalid {field_name}: {value}"))
}

fn update_status_in_table_file(
    path: &std::path::Path,
    tool: &str,
    to_status: &str,
) -> Result<bool> {
    let text = read_utf8(path)?;
    let mut updated = false;
    let mut out = Vec::new();
    let chunks = text.split("[[tools]]").collect::<Vec<_>>();
    if let Some(head) = chunks.first() {
        out.push((*head).to_string());
    }
    for chunk in chunks.iter().skip(1) {
        let mut block = format!("[[tools]]{chunk}");
        if block.contains(&format!("id = \"{tool}\""))
            || block.contains(&format!("tool_id = \"{tool}\""))
        {
            let mut lines = block.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
            if let Some(index) = lines
                .iter()
                .position(|line| line.trim_start().starts_with("status = "))
            {
                lines[index] = format!("status = \"{to_status}\"");
                updated = true;
            }
            block = format!("{}\n", lines.join("\n"));
        }
        out.push(block);
    }
    write_utf8(path, &out.concat())?;
    Ok(updated)
}

fn set_registry_status(paths: &[PathBuf], tool: &str, to_status: &str) -> Result<()> {
    let mut updated_any = false;
    for path in paths {
        updated_any |= update_status_in_table_file(path, tool, to_status)?;
    }
    if !updated_any {
        return Err(anyhow!("tool not found: {tool}"));
    }
    Ok(())
}

fn set_versions_status(workspace: &Workspace, tool: &str, to_status: &str) -> Result<()> {
    let path = versions_toml_path(workspace);
    let text = read_utf8(&path)?;
    let mut updated = false;
    let mut out = Vec::new();
    let chunks = text.split('[').collect::<Vec<_>>();
    if let Some(head) = chunks.first() {
        out.push((*head).to_string());
    }
    for chunk in chunks.iter().skip(1) {
        let block = format!("[{chunk}");
        let Some(table_end) = block.find(']') else {
            out.push(block);
            continue;
        };
        let table_name = block[1..table_end].trim();
        if table_name != tool {
            out.push(block);
            continue;
        }
        let mut lines = block.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
        if let Some(index) = lines
            .iter()
            .position(|line| line.trim_start().starts_with("status = "))
        {
            lines[index] = format!("status = \"{to_status}\"");
        } else {
            lines.insert(1, format!("status = \"{to_status}\""));
        }
        updated = true;
        out.push(format!("{}\n", lines.join("\n")));
    }
    if !updated {
        return Err(anyhow!("missing versions entry for {tool}"));
    }
    write_utf8(&path, &out.concat())
}

fn append_toml_table(path: &std::path::Path, content: &str, new_file_header: &str) -> Result<()> {
    let body = if path.exists() {
        format!("{}\n\n{}", read_utf8(path)?.trim_end(), content.trim_end())
    } else {
        format!("{}{}", new_file_header, content.trim_end())
    };
    write_utf8(path, &format!("{body}\n"))
}

fn iso_root_path(workspace: &Workspace) -> PathBuf {
    PathBuf::from(
        std::env::var("ISO_ROOT")
            .unwrap_or_else(|_| workspace.path("artifacts").display().to_string()),
    )
}

fn iso_run_id() -> String {
    env_or_default("ISO_RUN_ID", "run")
}

fn policy_path(workspace: &Workspace, env_key: &str, default_rel: &str) -> PathBuf {
    std::env::var(env_key).map_or_else(|_| workspace.path(default_rel), PathBuf::from)
}

fn apptainer_def_paths(workspace: &Workspace) -> Vec<PathBuf> {
    let mut paths = walkdir::WalkDir::new(workspace.path("containers/apptainer"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("def"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn tool_status_manifest(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let mut statuses = BTreeMap::new();
    for raw in read_utf8(&workspace.path("containers/TOOL_IDS.txt"))?.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((tool_id, status)) = line.split_once('\t') {
            statuses.insert(tool_id.to_string(), status.to_string());
        }
    }
    Ok(statuses)
}

fn images_metadata(workspace: &Workspace) -> Result<toml::map::Map<String, toml::Value>> {
    load_toml(&workspace.path("configs/ci/tools/images.toml"))?
        .as_table()
        .cloned()
        .ok_or_else(|| anyhow!("images.toml must be a TOML table"))
}

fn toolkit_bundles(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let value = load_toml(&workspace.path("configs/ci/tools/toolkit_bundles.toml"))?;
    let mut rows = BTreeMap::new();
    if let Some(table) = value.get("bundles").and_then(toml::Value::as_table) {
        for (bundle, row) in table {
            if let Some(row) = row.as_table() {
                rows.insert(bundle.clone(), row.clone());
            }
        }
    }
    Ok(rows)
}

fn docker_tool_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/docker/arm64").display()
            )
        })?
        .filter_map(std::result::Result::ok)
    {
        if let Some(tool) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix("Dockerfile."))
        {
            ids.insert(tool.to_string());
        }
    }
    Ok(ids)
}

fn dockerfile_paths(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/docker/arm64").display()
            )
        })?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("Dockerfile."))
        })
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn apptainer_tool_ids(workspace: &Workspace) -> BTreeSet<String> {
    apptainer_def_paths(workspace)
        .into_iter()
        .filter_map(|path| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn command_hostname() -> String {
    for args in [["-f"].as_slice(), [].as_slice()] {
        let mut command = std::process::Command::new("hostname");
        command.args(args);
        let Ok(output) = command.output() else {
            continue;
        };
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }
    String::new()
}

fn table_string(table: &toml::map::Map<String, toml::Value>, key: &str) -> String {
    table
        .get(key)
        .map(toml_value_string)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn table_bool(table: &toml::map::Map<String, toml::Value>, key: &str) -> bool {
    table
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn table_array_strings(table: &toml::map::Map<String, toml::Value>, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(toml_value_string)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn toml_value_string(value: &toml::Value) -> String {
    match value {
        toml::Value::String(value) => value.clone(),
        toml::Value::Integer(value) => value.to_string(),
        toml::Value::Float(value) => value.to_string(),
        toml::Value::Boolean(value) => value.to_string(),
        toml::Value::Datetime(value) => value.to_string(),
        toml::Value::Array(values) => values
            .iter()
            .map(toml_value_string)
            .collect::<Vec<_>>()
            .join(","),
        toml::Value::Table(_) => String::new(),
    }
}

fn markdown_code_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn has_shell_word(line: &str, word: &str) -> bool {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
        .any(|token| token == word)
}

fn line_has_network_command(line: &str) -> bool {
    let lowered = line.to_ascii_lowercase();
    lowered.contains("git clone")
        || lowered.contains("apt-get update")
        || has_shell_word(&lowered, "curl")
        || has_shell_word(&lowered, "wget")
}

fn read_json(path: &std::path::Path) -> Result<serde_json::Value> {
    serde_json::from_str(&read_utf8(path)?)
        .with_context(|| format!("parse JSON {}", path.display()))
}

fn json_string_pretty(value: &serde_json::Value) -> Result<String> {
    Ok(format!("{}\n", serde_json::to_string_pretty(value)?))
}

fn git_last_modified_timestamp(workspace: &Workspace, rel_path: &str) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(&workspace.root)
        .args(["log", "-1", "--format=%cI", "--", rel_path])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

fn out_path_arg(
    workspace: &Workspace,
    args: &[String],
    default_rel: &str,
    usage: &str,
) -> Result<PathBuf> {
    match args {
        [] => Ok(workspace.path(default_rel)),
        [single] if single == "--help" || single == "-h" => Err(anyhow!(usage.to_string())),
        [single] => Ok(path_from_arg(workspace, single)),
        _ => Err(anyhow!(usage.to_string())),
    }
}

fn path_from_arg(workspace: &Workspace, arg: &str) -> PathBuf {
    let path = PathBuf::from(arg);
    if path.is_absolute() {
        path
    } else {
        workspace.root.join(path)
    }
}

#[derive(Serialize)]
struct VersionMapItem {
    tool: String,
    version: String,
    status: String,
    source: String,
    source_sha256: String,
    pinned_commit: String,
    date_pinned: String,
}
