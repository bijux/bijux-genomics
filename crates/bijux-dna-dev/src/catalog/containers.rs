use anyhow::{anyhow, Result};

use crate::model::container::{
    ContainerCommandDefinition, ContainerCommandSpec, NativeContainerCommandKey,
};
use crate::runtime::workspace::Workspace;

pub fn container_registry(_workspace: &Workspace) -> Result<Vec<ContainerCommandDefinition>> {
    let mut commands = native_container_commands();
    commands.sort_by(|left, right| left.id.cmp(&right.id));

    for pair in commands.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(anyhow!("duplicate container command id `{}`", pair[0].id));
        }
    }

    Ok(commands)
}

#[allow(clippy::too_many_lines)]
fn native_container_commands() -> Vec<ContainerCommandDefinition> {
    vec![
        native(
            "lint",
            "Run the governed container lint surface.",
            NativeContainerCommandKey::Lint,
        ),
        native(
            "registry-tools",
            "Delegate container registry queries through the governed CLI.",
            NativeContainerCommandKey::RegistryTools,
        ),
        native(
            "ensure-images",
            "Plan or ensure governed container image coverage.",
            NativeContainerCommandKey::EnsureImages,
        ),
        native(
            "container-doctor",
            "Summarize governed container health and drift status.",
            NativeContainerCommandKey::ContainerDoctor,
        ),
        native(
            "release-gate",
            "Run the governed container release gate.",
            NativeContainerCommandKey::ReleaseGate,
        ),
        native(
            "vuln-scan-hook",
            "Generate the governed vulnerability scan hook report.",
            NativeContainerCommandKey::VulnScanHook,
        ),
        native(
            "apptainer-build-all",
            "Run the frontend apptainer build-and-proof workflow.",
            NativeContainerCommandKey::ApptainerBuildAll,
        ),
        native(
            "build-apptainer-all",
            "Build the selected apptainer definitions through the native environment surface.",
            NativeContainerCommandKey::BuildApptainerAll,
        ),
        native(
            "build-apptainer-hpc-frontend",
            "Build and compare frontend apptainer artifacts through the native workflow.",
            NativeContainerCommandKey::BuildApptainerHpcFrontend,
        ),
        native(
            "docker-build-all",
            "Run the docker-arm64 build-and-proof workflow.",
            NativeContainerCommandKey::DockerBuildAll,
        ),
        native(
            "smoke-apptainer",
            "Run the apptainer smoke surface.",
            NativeContainerCommandKey::SmokeApptainer,
        ),
        native(
            "smoke-docker-amd64",
            "Run the docker-amd64 smoke surface.",
            NativeContainerCommandKey::SmokeDockerAmd64,
        ),
        native(
            "smoke-docker-arm64",
            "Run the docker-arm64 smoke surface.",
            NativeContainerCommandKey::SmokeDockerArm64,
        ),
        native(
            "run-apptainer-frontend-smoke",
            "Run the frontend apptainer smoke proof workflow.",
            NativeContainerCommandKey::RunApptainerFrontendSmoke,
        ),
        native(
            "run-apptainer-frontend-security",
            "Run the frontend apptainer security workflow.",
            NativeContainerCommandKey::RunApptainerFrontendSecurity,
        ),
        native(
            "run-apptainer-frontend-reproducibility",
            "Run the frontend apptainer reproducibility workflow.",
            NativeContainerCommandKey::RunApptainerFrontendReproducibility,
        ),
        native(
            "container-runtime-check",
            "Print the selected runtime contract inputs.",
            NativeContainerCommandKey::ContainerRuntimeCheck,
        ),
        native(
            "generate-tool-ids",
            "Generate the authoritative container tool id manifest.",
            NativeContainerCommandKey::GenerateToolIds,
        ),
        native(
            "check-tool-id-manifest",
            "Validate the generated container tool id manifest.",
            NativeContainerCommandKey::CheckToolIdManifest,
        ),
        native(
            "generate-tool-name-map",
            "Generate the tool id to expected binary mapping document.",
            NativeContainerCommandKey::GenerateToolNameMap,
        ),
        native(
            "check-tool-name-map-generated",
            "Validate the generated tool name mapping document.",
            NativeContainerCommandKey::CheckToolNameMapGenerated,
        ),
        native(
            "generate-index",
            "Generate the container docs index from registry and file coverage.",
            NativeContainerCommandKey::GenerateContainerIndex,
        ),
        native(
            "check-index",
            "Validate the generated container docs index.",
            NativeContainerCommandKey::CheckContainerIndex,
        ),
        native(
            "generate-ghcr-publish-matrix",
            "Generate the GHCR publication matrix for governed Docker container images.",
            NativeContainerCommandKey::GenerateGhcrPublishMatrix,
        ),
        native(
            "generate-license-metadata",
            "Generate container license metadata and the license index document.",
            NativeContainerCommandKey::GenerateLicenseMetadata,
        ),
        native(
            "check-license-metadata",
            "Validate generated container license metadata files.",
            NativeContainerCommandKey::CheckLicenseMetadata,
        ),
        native(
            "check-license-index-generated",
            "Validate the generated container license index document.",
            NativeContainerCommandKey::CheckLicenseIndexGenerated,
        ),
        native(
            "generate-qa-matrix",
            "Generate the apptainer QA matrix from registry and artifact metadata.",
            NativeContainerCommandKey::GenerateQaMatrix,
        ),
        native(
            "check-qa-matrix-generated",
            "Validate the generated apptainer QA matrix.",
            NativeContainerCommandKey::CheckQaMatrixGenerated,
        ),
        native(
            "generate-tool-docs",
            "Generate per-tool container contract documents.",
            NativeContainerCommandKey::GenerateToolDocs,
        ),
        native(
            "check-tool-docs-generated",
            "Validate the generated per-tool container contract documents.",
            NativeContainerCommandKey::CheckToolDocsGenerated,
        ),
        native(
            "generate-network-usage",
            "Generate the container network usage inventory.",
            NativeContainerCommandKey::GenerateNetworkUsage,
        ),
        native(
            "check-network-disclosure",
            "Validate container network disclosure metadata and offline policy.",
            NativeContainerCommandKey::CheckNetworkDisclosure,
        ),
        native(
            "extract-version-map",
            "Generate the normalized version map from versions.toml.",
            NativeContainerCommandKey::ExtractVersionMap,
        ),
        native(
            "generate-version-lock",
            "Generate the governed container version lock file.",
            NativeContainerCommandKey::GenerateVersionLock,
        ),
        native(
            "check-version-lock",
            "Validate the generated container version lock file.",
            NativeContainerCommandKey::CheckVersionLock,
        ),
        native(
            "check-version-authority",
            "Validate the canonical version and lock authority contracts.",
            NativeContainerCommandKey::CheckVersionAuthority,
        ),
        native(
            "generate-versions-index-sha",
            "Generate the checksum index for files under containers/versions.",
            NativeContainerCommandKey::GenerateVersionsIndexSha,
        ),
        native(
            "check-versions-index-sha",
            "Validate the checksum index for files under containers/versions.",
            NativeContainerCommandKey::CheckVersionsIndexSha,
        ),
        native(
            "check-lock-change-discipline",
            "Validate that versions.toml and lock.json change together in CI history.",
            NativeContainerCommandKey::CheckLockChangeDiscipline,
        ),
        native(
            "check-lock-drift",
            "Validate the generated container version lock file.",
            NativeContainerCommandKey::CheckLockDrift,
        ),
        native(
            "check-lock-schema",
            "Validate the schema contract for containers/versions/lock.json.",
            NativeContainerCommandKey::CheckLockSchema,
        ),
        native(
            "check-version-completeness",
            "Validate that every governed container has a versions.toml entry.",
            NativeContainerCommandKey::CheckVersionCompleteness,
        ),
        native(
            "check-version-hash-pin",
            "Validate that version entries record concrete provenance pins.",
            NativeContainerCommandKey::CheckVersionHashPin,
        ),
        native(
            "check-version-immutability",
            "Validate that production version pins are immutable across CI commits.",
            NativeContainerCommandKey::CheckVersionImmutability,
        ),
        native(
            "check-version-deprecations",
            "Validate container version deprecation metadata against the lock and version map.",
            NativeContainerCommandKey::CheckVersionDeprecations,
        ),
        native(
            "check-promotion-policy",
            "Validate promotion policy documentation markers and native command references.",
            NativeContainerCommandKey::CheckPromotionPolicy,
        ),
        native(
            "check-promotion-lock-integrity",
            "Validate that production tools remain represented by canonical lock metadata.",
            NativeContainerCommandKey::CheckPromotionLockIntegrity,
        ),
        native(
            "promote",
            "Change a tool lifecycle status and regenerate governed container metadata.",
            NativeContainerCommandKey::Promote,
        ),
        native(
            "demote",
            "Demote a production tool and append registry deprecation metadata.",
            NativeContainerCommandKey::Demote,
        ),
        native(
            "deprecate-version",
            "Append a container version deprecation entry and regenerate governed metadata.",
            NativeContainerCommandKey::DeprecateVersion,
        ),
        native(
            "tool-lifecycle",
            "Apply lifecycle aliases for experimental and stable container states.",
            NativeContainerCommandKey::ToolLifecycle,
        ),
        native(
            "check-apptainer-cache-policy",
            "Validate the governed Apptainer cache policy wiring.",
            NativeContainerCommandKey::CheckApptainerCachePolicy,
        ),
        native(
            "check-apptainer-frontend-reproducibility",
            "Validate frontend Apptainer reproducibility results against policy.",
            NativeContainerCommandKey::CheckApptainerFrontendReproducibility,
        ),
        native(
            "check-apptainer-frontend-security",
            "Validate frontend Apptainer security summary results against policy.",
            NativeContainerCommandKey::CheckApptainerFrontendSecurity,
        ),
        native(
            "check-apptainer-frontend-smoke-proof",
            "Validate frontend Apptainer smoke proof outputs and policy flags.",
            NativeContainerCommandKey::CheckApptainerFrontendSmokeProof,
        ),
        native(
            "check-apptainer-frontend-version-output-lock",
            "Validate frontend smoke version output hashes against the governed lock.",
            NativeContainerCommandKey::CheckApptainerFrontendVersionOutputLock,
        ),
        native(
            "check-apptainer-hardening",
            "Validate Apptainer definition hardening and label contracts.",
            NativeContainerCommandKey::CheckApptainerHardening,
        ),
        native(
            "check-apptainer-post-pins",
            "Validate Apptainer %post download pinning and compute-node policy.",
            NativeContainerCommandKey::CheckApptainerPostPins,
        ),
        native(
            "check-apptainer-version-label-sync",
            "Validate Apptainer version labels against versions.toml.",
            NativeContainerCommandKey::CheckApptainerVersionLabelSync,
        ),
        native(
            "check-bijux-apptainer-built",
            "Validate that governed Bijux Apptainer images were built and recorded.",
            NativeContainerCommandKey::CheckBijuxApptainerBuilt,
        ),
        native(
            "generate-local-apptainer-digests",
            "Generate local Apptainer SIF digests for frontend parity checks.",
            NativeContainerCommandKey::GenerateLocalApptainerDigests,
        ),
        native(
            "compare-frontend-local-sif-hash",
            "Compare frontend and local Apptainer SIF digests and write a diff report.",
            NativeContainerCommandKey::CompareFrontendLocalSifHash,
        ),
        native(
            "check-missing-images",
            "Validate that governed tools and bundles resolve to concrete image coverage.",
            NativeContainerCommandKey::CheckMissingImages,
        ),
        native(
            "check-non-bijux-sources",
            "Validate NON_BIJUX_SOURCES.md coverage for upstream-derived Apptainer definitions.",
            NativeContainerCommandKey::CheckNonBijuxSources,
        ),
        native(
            "check-owners",
            "Validate explicit ownership coverage for every governed container tool.",
            NativeContainerCommandKey::CheckOwners,
        ),
        native(
            "check-registry-vs-defs",
            "Validate registry container declarations against concrete Dockerfile and Apptainer defs.",
            NativeContainerCommandKey::CheckRegistryVsDefs,
        ),
        native(
            "check-tool-name-collision",
            "Validate tool id normalization, name-map parity, and expected binary collisions.",
            NativeContainerCommandKey::CheckToolNameCollision,
        ),
        native(
            "check-tool-container-coverage",
            "Validate production registry container tools against Docker and Apptainer coverage policy.",
            NativeContainerCommandKey::CheckToolContainerCoverage,
        ),
        native(
            "check-toolkit-bundles",
            "Validate toolkit bundle tool coverage against registry and image metadata.",
            NativeContainerCommandKey::CheckToolkitBundles,
        ),
        native(
            "check-hpc-image-naming",
            "Validate HPC image naming against the ensure-images plan report and naming policy.",
            NativeContainerCommandKey::CheckHpcImageNaming,
        ),
        native(
            "check-planned-actionability",
            "Validate that PLANNED.md retains actionable rows and explicit ownership.",
            NativeContainerCommandKey::CheckPlannedActionability,
        ),
        native(
            "check-bijux-template-markers",
            "Validate template markers across Bijux-owned Apptainer definitions.",
            NativeContainerCommandKey::CheckBijuxTemplateMarkers,
        ),
        native(
            "check-tool-id-contract",
            "Validate the generated tool id manifest contract against concrete container mappings.",
            NativeContainerCommandKey::CheckToolIdContract,
        ),
        native(
            "check-docker-arch-policy",
            "Validate the arm64-only Docker policy and multiarch planning documentation.",
            NativeContainerCommandKey::CheckDockerArchPolicy,
        ),
        native(
            "check-docker-arm64-completeness",
            "Validate docker-arm64 coverage for every registry tool that declares Docker runtime support.",
            NativeContainerCommandKey::CheckDockerArm64Completeness,
        ),
        native(
            "check-docker-context",
            "Validate Docker build context minimization and forbidden broad copies.",
            NativeContainerCommandKey::CheckDockerContext,
        ),
        native(
            "check-docker-hardening",
            "Validate Dockerfile hardening, non-root, and entrypoint contracts.",
            NativeContainerCommandKey::CheckDockerHardening,
        ),
        native(
            "check-docker-labels",
            "Validate Docker label coverage and version parity with Apptainer definitions.",
            NativeContainerCommandKey::CheckDockerLabels,
        ),
        native(
            "check-docker-unpinned-apt",
            "Validate apt package pinning across Dockerfiles.",
            NativeContainerCommandKey::CheckDockerUnpinnedApt,
        ),
        native(
            "check-docker-version-sync",
            "Validate Docker TOOL_VERSION args and image version labels against versions.toml.",
            NativeContainerCommandKey::CheckDockerVersionSync,
        ),
        native(
            "check-dockerfiles-built",
            "Validate docker-arm64 build manifests for every governed Dockerfile in CI.",
            NativeContainerCommandKey::CheckDockerfilesBuilt,
        ),
        native(
            "check-no-secrets",
            "Scan container recipes for committed secret patterns.",
            NativeContainerCommandKey::CheckNoSecrets,
        ),
        native(
            "check-runtime-downloads",
            "Validate runtime download policy across recipe entrypoints and runtime commands.",
            NativeContainerCommandKey::CheckRuntimeDownloads,
        ),
        native(
            "check-vuln-allowlist",
            "Validate vulnerability allowlist formatting, uniqueness, and expiry windows.",
            NativeContainerCommandKey::CheckVulnAllowlist,
        ),
        native(
            "check-vuln-hook",
            "Validate promoted-tool vulnerability hook coverage and report artifacts.",
            NativeContainerCommandKey::CheckVulnHook,
        ),
        native(
            "check-sbom-artifacts",
            "Validate SBOM artifacts and smoke-log evidence for promoted tool manifests.",
            NativeContainerCommandKey::CheckSbomArtifacts,
        ),
        native(
            "check-time-locale-determinism",
            "Validate deterministic TZ and locale wiring across container runtimes.",
            NativeContainerCommandKey::CheckTimeLocaleDeterminism,
        ),
        native(
            "check-tool-invocation-normalization",
            "Validate smoke command prefixes against expected tool binary names.",
            NativeContainerCommandKey::CheckToolInvocationNormalization,
        ),
        native(
            "check-smoke-inputs-policy",
            "Validate smoke input fixtures declared in smoke_inputs_policy.toml.",
            NativeContainerCommandKey::CheckSmokeInputsPolicy,
        ),
        native(
            "check-cross-runtime-representative",
            "Validate representative version parity across Docker and Apptainer smoke outputs.",
            NativeContainerCommandKey::CheckCrossRuntimeRepresentative,
        ),
        native(
            "check-cross-runtime-smoke",
            "Validate cross-runtime smoke parity for shared Docker and Apptainer tools.",
            NativeContainerCommandKey::CheckCrossRuntimeSmoke,
        ),
        native(
            "check-smoke-failure-classification",
            "Validate smoke manifests record governed failure classes for failed runs.",
            NativeContainerCommandKey::CheckSmokeFailureClassification,
        ),
        native(
            "check-smoke-contract",
            "Validate registry smoke command contracts for every governed container tool.",
            NativeContainerCommandKey::CheckSmokeContract,
        ),
        native(
            "check-smoke-contract-lock",
            "Validate frontend smoke summaries against the governed container lock.",
            NativeContainerCommandKey::CheckSmokeContractLock,
        ),
        native(
            "check-vcf-imputation-toolchain",
            "Validate the governed VCF imputation toolchain across registry, docs, and container metadata.",
            NativeContainerCommandKey::CheckVcfImputationToolchain,
        ),
        native(
            "check-imputation-runtime-constraints",
            "Validate documented runtime constraints for governed VCF imputation tools.",
            NativeContainerCommandKey::CheckImputationRuntimeConstraints,
        ),
        native(
            "check-imputation-network-policy",
            "Validate network metadata for governed VCF imputation tools.",
            NativeContainerCommandKey::CheckImputationNetworkPolicy,
        ),
        native(
            "check-imputation-hardening",
            "Validate Docker hardening exceptions and entrypoint coverage for governed VCF imputation tools.",
            NativeContainerCommandKey::CheckImputationHardening,
        ),
        native(
            "check-imputation-release-smoke",
            "Validate Docker and Apptainer release smoke summaries for governed VCF imputation tools.",
            NativeContainerCommandKey::CheckImputationReleaseSmoke,
        ),
        native(
            "check-imputation-cross-runtime-parity",
            "Validate version parity across Docker and Apptainer for governed VCF imputation tools.",
            NativeContainerCommandKey::CheckImputationCrossRuntimeParity,
        ),
        native(
            "check-build-provenance",
            "Validate provenance metadata across governed downstream container definitions and manifests.",
            NativeContainerCommandKey::CheckBuildProvenance,
        ),
        native(
            "check-digest-changes-on-version-change",
            "Validate that lock digests change when governed versions change across commits.",
            NativeContainerCommandKey::CheckDigestChangesOnVersionChange,
        ),
        native(
            "check-digest-output-policy",
            "Validate digest artifact placement and forbid floating latest references in governed docs.",
            NativeContainerCommandKey::CheckDigestOutputPolicy,
        ),
        native(
            "check-runtime-tool-digest-recording",
            "Validate runtime contracts that record resolved tool digests.",
            NativeContainerCommandKey::CheckRuntimeToolDigestRecording,
        ),
        native(
            "check-rebuild-repro",
            "Rebuild a Docker tool twice and validate reproducible version and provenance output.",
            NativeContainerCommandKey::CheckRebuildRepro,
        ),
        native(
            "check-apptainer-rebuild-repro",
            "Rebuild an Apptainer tool twice and validate reproducible SIF output.",
            NativeContainerCommandKey::CheckApptainerRebuildRepro,
        ),
        native(
            "check-apptainer-bijux-header",
            "Validate the governed Bijux header on every Apptainer definition.",
            NativeContainerCommandKey::CheckApptainerBijuxHeader,
        ),
        native(
            "check-hpc-frontend-policy-enforcement",
            "Validate compute-node refusal and pin enforcement across HPC frontend scripts.",
            NativeContainerCommandKey::CheckHpcFrontendPolicyEnforcement,
        ),
        native(
            "check-image-size-regression",
            "Validate promoted image-size growth against the governed acknowledgement policy.",
            NativeContainerCommandKey::CheckImageSizeRegression,
        ),
        native(
            "check-lock-matches-built-output",
            "Validate built container outputs against the governed lock contract.",
            NativeContainerCommandKey::CheckLockMatchesBuiltOutput,
        ),
        native(
            "check-release-checklist",
            "Validate the release checklist mappings against the release gate entrypoint.",
            NativeContainerCommandKey::CheckReleaseChecklist,
        ),
        native(
            "check-toolkit-bundle-buildable",
            "Validate toolkit bundles include at least one governed buildable tool.",
            NativeContainerCommandKey::CheckToolkitBundleBuildable,
        ),
        native(
            "check-vcf-downstream-bundle-coverage",
            "Validate the VCF downstream toolkit bundle covers phasing and imputation stages.",
            NativeContainerCommandKey::CheckVcfDownstreamBundleCoverage,
        ),
        native(
            "summary",
            "Summarize container manifests and optionally write JSON output.",
            NativeContainerCommandKey::Summary,
        ),
        native(
            "env-prep",
            "Prepare tool or stage environments for the selected container runtime.",
            NativeContainerCommandKey::EnvPrep,
        ),
        native(
            "env-smoke",
            "Run environment smoke checks for the selected container runtime.",
            NativeContainerCommandKey::EnvSmoke,
        ),
        native(
            "container-smoke",
            "Prepare and smoke a single tool or stage in the selected runtime.",
            NativeContainerCommandKey::ContainerSmoke,
        ),
        native(
            "containers-smoke",
            "Smoke every registered stage in the selected runtime.",
            NativeContainerCommandKey::ContainersSmoke,
        ),
        native(
            "smoke-containers-docker-arm64",
            "Run the docker-arm64 smoke surface with the current tool selection.",
            NativeContainerCommandKey::SmokeContainersDockerArm64,
        ),
        native(
            "smoke-containers-docker-amd64",
            "Run the docker-amd64 smoke surface with the current tool selection.",
            NativeContainerCommandKey::SmokeContainersDockerAmd64,
        ),
        native(
            "smoke-containers-apptainer",
            "Run the apptainer smoke surface with the current tool selection.",
            NativeContainerCommandKey::SmokeContainersApptainer,
        ),
        native(
            "smoke-cntainers-apptainer-bijux-run",
            "Run apptainer smoke through the bijux-run execution path.",
            NativeContainerCommandKey::SmokeCntainersApptainerBijuxRun,
        ),
        native(
            "smoke-cntainers-apptainer-apptainer-run",
            "Run apptainer smoke through the direct apptainer-run path.",
            NativeContainerCommandKey::SmokeCntainersApptainerApptainerRun,
        ),
        native(
            "smoke-cntainers-apptainer-verify",
            "Compare apptainer smoke outputs across execution paths.",
            NativeContainerCommandKey::SmokeCntainersApptainerVerify,
        ),
        native(
            "smoke-cross-runtime-verify",
            "Compare docker and apptainer smoke outputs.",
            NativeContainerCommandKey::SmokeCrossRuntimeVerify,
        ),
        native(
            "smoke-toolkit-docker-arm64",
            "Smoke a toolkit bundle with docker-arm64.",
            NativeContainerCommandKey::SmokeToolkitDockerArm64,
        ),
        native(
            "smoke-toolkit-apptainer",
            "Smoke a toolkit bundle with apptainer.",
            NativeContainerCommandKey::SmokeToolkitApptainer,
        ),
        native(
            "build-images",
            "Build the current tool selection for the chosen runtime.",
            NativeContainerCommandKey::BuildImages,
        ),
        native(
            "build-tool",
            "Build a single tool image for the chosen runtime.",
            NativeContainerCommandKey::BuildTool,
        ),
        native(
            "build-all",
            "Build all primary tool images for the chosen runtime.",
            NativeContainerCommandKey::BuildAll,
        ),
        native(
            "build-bundle",
            "Build all images in the selected toolkit bundle.",
            NativeContainerCommandKey::BuildBundle,
        ),
        native(
            "test-images",
            "Run the standard image test surface for the chosen runtime.",
            NativeContainerCommandKey::TestImages,
        ),
        native(
            "test-images-stage",
            "Run image tests for a single stage.",
            NativeContainerCommandKey::TestImagesStage,
        ),
        native(
            "test-images-tool",
            "Run image tests for a single tool.",
            NativeContainerCommandKey::TestImagesTool,
        ),
        native(
            "image-smoke-vcf",
            "Smoke the VCF image surface assembled from the stage registry.",
            NativeContainerCommandKey::ImageSmokeVcf,
        ),
        native(
            "image-qa",
            "Run the governed image QA workflow.",
            NativeContainerCommandKey::ImageQa,
        ),
        native(
            "apptainer-ensure",
            "Ensure the requested apptainer images exist on the frontend.",
            NativeContainerCommandKey::ApptainerEnsure,
        ),
        native(
            "apptainer-ensure-stage",
            "Ensure the requested apptainer stage images exist on the frontend.",
            NativeContainerCommandKey::ApptainerEnsureStage,
        ),
    ]
}

fn native(
    id: &'static str,
    summary: &'static str,
    key: NativeContainerCommandKey,
) -> ContainerCommandDefinition {
    ContainerCommandDefinition {
        id: id.to_string(),
        summary: summary.to_string(),
        command: ContainerCommandSpec::Native { key },
    }
}
